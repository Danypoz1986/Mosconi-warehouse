
const productsList = document.getElementById("products-list");

let editingProductId = null;

const productForm = document.getElementById("product-form");
const formTitle = document.querySelector("section:has(#product-form) h2");
const submitButton = productForm.querySelector('button[type="submit"]');
const cancelButton = document.getElementById("cancel-button");

// Load products from the Rust API
async function loadProducts() {
    try {
        const response = await fetch("/products");

        if (!response.ok) {
            throw new Error("Failed to load products");
        }

        const products = await response.json();

        // Clear the existing table rows
        productsList.replaceChildren();

        // Display a message if the inventory is empty
        if (products.length === 0) {
            const row = document.createElement("tr");
            const cell = document.createElement("td");

            cell.colSpan = 5;
            cell.textContent = "No products available.";

            row.appendChild(cell);
            productsList.appendChild(row);
            return;
        }

        // Create a table row for each product
        products.forEach((product) => {
            const row = document.createElement("tr");

            const values = [
                product.id,
                product.name,
                product.quantity,
                Number(product.price).toFixed(2)
            ];

            values.forEach((value) => {
                const cell = document.createElement("td");
                cell.textContent = value;
                row.appendChild(cell);
            });

            // Create the Edit button
            const actionsCell = document.createElement("td");
            const editButton = document.createElement("button");

            editButton.textContent = "Edit";
            editButton.type = "button";

            // Fill the form with the selected product data
            editButton.addEventListener("click", () => {
                editingProductId = product.id;

                document.getElementById("name").value = product.name;
                document.getElementById("quantity").value = product.quantity;
                document.getElementById("price").value = product.price;

                formTitle.textContent = "Edit Product";
                submitButton.textContent = "Save Changes";

                // Show the Cancel button during editing
                cancelButton.hidden = false;

                productForm.scrollIntoView({ behavior: "smooth" });
            });

            actionsCell.appendChild(editButton);

            // Create the Delete button
            const deleteButton = document.createElement("button");

            deleteButton.textContent = "Delete";
            deleteButton.type = "button";
            deleteButton.classList.add("delete-button");

            // Delete the selected product after confirmation
            deleteButton.addEventListener("click", async () => {
                const confirmed = confirm(
                    `Are you sure you want to delete "${product.name}"?`
                );

                if (!confirmed) {
                    return;
                }

                try {
                    // Send the DELETE request to the Rust API
                    const response = await fetch(`/products/${product.id}`, {
                        method: "DELETE"
                    });

                    if (!response.ok) {
                        throw new Error("Failed to delete product");
                    }

                    // Reset the form if the deleted product was being edited
                    if (editingProductId === product.id) {
                        productForm.reset();
                        editingProductId = null;

                        formTitle.textContent = "Add Product";
                        submitButton.textContent = "Add Product";
                        cancelButton.hidden = true;
                    }

                    // Refresh the products table
                    await loadProducts();

                    // Display a success message
                    alert("Product deleted successfully!");

                } catch (error) {
                    console.error("Error deleting product:", error);
                    alert("Unable to delete product.");
                }
            });

            actionsCell.appendChild(deleteButton);
            row.appendChild(actionsCell);

            productsList.appendChild(row);
        });

    } catch (error) {
        console.error("Error loading products:", error);

        const row = document.createElement("tr");
        const cell = document.createElement("td");

        cell.colSpan = 5;
        cell.textContent = "Unable to load products.";

        row.appendChild(cell);
        productsList.replaceChildren(row);
    }
}

// Load products when the page opens
loadProducts();

// Handle product creation and updates
productForm.addEventListener("submit", async (event) => {
    event.preventDefault();

    // Read the form values
    const name = document.getElementById("name").value.trim();
    const quantity = Number(document.getElementById("quantity").value);
    const price = Number(document.getElementById("price").value);

    const product = {
        name,
        quantity,
        price
    };

    try {
        // Use PUT when editing, POST when creating
        const isEditing = editingProductId !== null;

        const url = isEditing
            ? `/products/${editingProductId}`
            : "/products";

        // Send the product data to the Rust API
        const response = await fetch(url, {
            method: isEditing ? "PUT" : "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify(product)
        });

        if (!response.ok) {
            throw new Error(
                isEditing
                    ? "Failed to update product"
                    : "Failed to create product"
            );
        }

        // Reset the form after a successful request
        productForm.reset();
        editingProductId = null;

        formTitle.textContent = "Add Product";
        submitButton.textContent = "Add Product";
        cancelButton.hidden = true;

        // Refresh the products table
        await loadProducts();

        // Display a success message
        alert(isEditing
            ? "Product updated successfully!"
            : "Product added successfully!"
        );

    } catch (error) {
        console.error("Error saving product:", error);
        alert("Unable to save product.");
    }
});

// Cancel editing without saving changes
cancelButton.addEventListener("click", () => {
    productForm.reset();
    editingProductId = null;

    formTitle.textContent = "Add Product";
    submitButton.textContent = "Add Product";
    cancelButton.hidden = true;
});
