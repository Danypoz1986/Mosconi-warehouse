use postgres::{Client, NoTls};

// Prompt for a new product, validate its fields, and save it to PostgreSQL.
fn add_product(client: &mut Client) {
    // Read the product name from standard input.
    let mut name = String::new();
    println!("\nEnter product name:");
    std::io::stdin()
        .read_line(&mut name)
        .expect("Failed to read name");

    // Reject blank or whitespace-only product names.
    if name.trim().is_empty() {
        println!("\nProduct name cannot be empty!\n");
        return;
    }

    // Read the quantity from standard input.
    let mut quantity = String::new();
    println!("Enter quantity: ");
    std::io::stdin()
        .read_line(&mut quantity)
        .expect("Failed to read quantity");

    // Parse the quantity and reject negative or non-numeric input immediately.
    let quantity: i32 = match quantity.trim().parse() {
        Ok(value) if value >= 0 => value,
        _ => {
            println!("\nInvalid quantity! Enter a non-negative number.\n");
            return;
        }
    };

    // Read the product price from standard input.
    let mut price = String::new();
    println!("Enter price:");
    std::io::stdin()
        .read_line(&mut price)
        .expect("Failed to read price");

    // Validate the price and convert it to a floating-point number.

    // Reject negative, non-finite, or out-of-range prices.
    let price: f64 = match price.trim().parse::<f64>() {
        Ok(value) if value >= 0.0 && value <= 99_999_999.99 && value.is_finite() => value,
        _ => {
            println!("\nInvalid price! Enter a non-negative number.\n");
            return;
        }
    };

    // Insert the validated product into PostgreSQL.
    client
        .execute(
            "INSERT INTO products (name, quantity, price) VALUES ($1, $2, $3::float8::numeric)",
            &[&name.trim(), &quantity, &price],
        )
        .expect("Failed to insert product");

    println!("\nProduct added successfully!\n");
}

// Fetch and print the current inventory.
fn list_products(client: &mut Client) {
    // Retrieve all products from PostgreSQL, sorted by ID.
    let rows = client
        .query(
            "SELECT id, name, quantity, price::float8 FROM products ORDER BY id",
            &[],
        )
        .expect("Failed to fetch products");

    // Return to the menu when there are no products.
    if rows.is_empty() {
        println!("\nInventory is empty\n");
        return;
    }

    // Display each product returned by the query.
    println!("\n=== Product List ===\n");

    for row in rows {
        let id: i32 = row.get(0);
        let name: String = row.get(1);
        let quantity: i32 = row.get(2);
        let price: f64 = row.get(3);

        println!("ID: {}", id);
        println!("Product: {}", name);
        println!("Quantity: {}", quantity);
        println!("Price: {:.2}\n", price);
    }
}

// Look up a single product using its database ID.
fn search_product(client: &mut Client) {
    // Read the ID of the product to search for.
    let mut id = String::new();

    println!("Enter product ID:");
    std::io::stdin()
        .read_line(&mut id)
        .expect("Failed to read ID");

    // Accept only positive numeric IDs.
    let id: i32 = match id.trim().parse() {
        Ok(value) if value > 0 => value,
        _ => {
            println!("\nInvalid ID! Enter a positive number.\n");
            return;
        }
    };

    // Find the selected product by its ID in PostgreSQL.
    let row = client
        .query_opt(
            "SELECT id, name, quantity, price::float8 FROM products WHERE id = $1",
            &[&id],
        )
        .expect("Failed to search product");

    // Display the product details or a not-found message.
    match row {
        Some(product) => {
            println!("\nProduct found!\n");
            println!("ID: {}", product.get::<_, i32>(0));
            println!("Product: {}", product.get::<_, String>(1));
            println!("Quantity: {}", product.get::<_, i32>(2));
            println!("Price: {:.2}\n", product.get::<_, f64>(3));
        }
        None => println!("\nProduct not found!\n"),
    }
}

// Let the user update one field of an existing product.
fn update_product(client: &mut Client) {
    // Check whether any products exist before offering an update.
    let rows = client
        .query("SELECT id FROM products", &[])
        .expect("Failed to fetch products");

    if rows.is_empty() {
        println!("\nInventory is empty!\n");
        return;
    }

    list_products(client);

    // Read the ID of the product to update.
    let mut id = String::new();
    println!("Enter product ID to update:");
    std::io::stdin()
        .read_line(&mut id)
        .expect("Failed to read ID");

    // Accept only positive numeric IDs.
    let id: i32 = match id.trim().parse() {
        Ok(value) if value > 0 => value,
        _ => {
            println!("\nInvalid ID! Enter a positive number.\n");
            return;
        }
    };

    // Find the selected product by its ID in PostgreSQL.
    let row = client
        .query_opt(
            "SELECT id, name, quantity, price::float8 FROM products WHERE id = $1",
            &[&id],
        )
        .expect("Failed to search product");

    match row {
        Some(product) => {
            println!("\nProduct found!\n");
            println!("ID: {}", product.get::<_, i32>(0));
            println!("Product: {}", product.get::<_, String>(1));
            println!("Quantity: {}", product.get::<_, i32>(2));
            println!("Price: {:.2}\n", product.get::<_, f64>(3));

            loop {
                let mut choice = String::new();
                println!("1. Update product name");
                println!("2. Update product quantity");
                println!("3. Update product price");
                println!("0. Cancel update");

                std::io::stdin()
                    .read_line(&mut choice)
                    .expect("Failed to read input");

                match choice.trim() {
                    "0" => {
                        println!("\nUpdate cancelled");
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        return;
                    }
                    "1" => {
                        let mut new_name = String::new();
                        println!("Enter new product name:");
                        std::io::stdin()
                            .read_line(&mut new_name)
                            .expect("Failed to read input");

                        // Do not overwrite the name with blank text.
                        if new_name.trim().is_empty() {
                            println!("\nProduct name cannot be empty!\n");
                            return;
                        }

                        client
                            .execute(
                                "UPDATE products SET name = $1 WHERE id = $2",
                                &[&new_name.trim(), &id],
                            )
                            .expect("Failed to update product name");

                        println!("Product name updated successfully!");
                        return;
                    }
                    "2" => {
                        let mut new_quantity = String::new();
                        println!("Enter new product quantity:");
                        std::io::stdin()
                            .read_line(&mut new_quantity)
                            .expect("Failed to read input");

                        // Validate the new quantity before updating the database.
                        let new_quantity: i32 = match new_quantity.trim().parse() {
                            Ok(value) if value >= 0 => value,
                            _ => {
                                println!("\nInvalid quantity! Enter a non-negative number.\n");
                                return;
                            }
                        };

                        client
                            .execute(
                                "UPDATE products SET quantity = $1 WHERE id = $2",
                                &[&new_quantity, &id],
                            )
                            .expect("Failed to update product quantity");

                        println!("Product quantity updated successfully!");
                        return;
                    }
                    "3" => {
                        let mut new_price = String::new();
                        println!("Enter new product price:");
                        std::io::stdin()
                            .read_line(&mut new_price)
                            .expect("Failed to read input");

                        // Validate the new price against the database limits.
                        let new_price: f64 = match new_price.trim().parse::<f64>() {
                            Ok(value)
                                if value >= 0.0 && value <= 99_999_999.99 && value.is_finite() =>
                            {
                                value
                            }
                            _ => {
                                println!("\nInvalid price! Enter a non-negative number.\n");
                                return;
                            }
                        };

                        client
                            .execute(
                                "UPDATE products SET price = $1::float8::numeric WHERE id = $2",
                                &[&new_price, &id],
                            )
                            .expect("Failed to update product price");

                        println!("Product price updated successfully!");
                        return;
                    }
                    _ => println!("Invalid option. Enter your choice again:"),
                }
            }
        }
        None => println!("\nProduct not found!\n"),
    }
}

// Remove a product by ID after displaying the inventory.
fn delete_product(client: &mut Client) {
    // Check whether any products exist before offering deletion.
    let rows = client
        .query("SELECT id FROM products", &[])
        .expect("Failed to fetch products");

    if rows.is_empty() {
        println!("\nInventory is empty\n");
        return;
    }

    list_products(client);

    // Ask which product should be deleted.
    println!("Enter the ID of the product you want to delete:");

    let mut delete_id = String::new();
    std::io::stdin()
        .read_line(&mut delete_id)
        .expect("Failed to read input");

    // Accept only positive numeric IDs for deletion.
    let delete_id: i32 = match delete_id.trim().parse() {
        Ok(value) if value > 0 => value,
        _ => {
            println!("\nInvalid ID! Enter a positive number.\n");
            return;
        }
    };

    // Delete the matching row and check whether a row was affected.
    let deleted = client
        .execute("DELETE FROM products WHERE id = $1", &[&delete_id])
        .expect("Failed to delete product");

    if deleted > 0 {
        println!("\nProduct deleted successfully!\n");
    } else {
        println!("\nProduct not found!\n");
    }
}

// Print the available commands in the console menu.
fn show_menu() {
    println!("\n===== MOSCONI WAREHOUSE =====");
    println!("1. Add product");
    println!("2. List products");
    println!("3. Search product");
    println!("4. Update product");
    println!("5. Delete product");
    println!("0. Exit");
    println!("Select an option:");
}

// Initialize the database connection and run the interactive menu.
fn main() {
    // Load environment variables from the .env file.
    dotenvy::dotenv().expect("Failed to load .env file");

    // Read the PostgreSQL connection string from the environment.
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not found");

    // Establish a connection to the PostgreSQL database.
    let mut client =
        Client::connect(&database_url, NoTls).expect("Failed to connect to PostgreSQL");

    println!("\nConnected to PostgreSQL successfully!");

    // Count the products currently stored in the database.
    let row = client
        .query_one("SELECT COUNT(*) FROM products", &[])
        .expect("Failed to query products");

    let count: i64 = row.get(0);

    println!("\nProducts in database: {}", count);

    // Keep showing the menu until the user chooses to exit.
    loop {
        show_menu();

        let mut choice = String::new();
        std::io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read input");

        match choice.trim() {
            "1" => add_product(&mut client),
            "2" => list_products(&mut client),
            "3" => search_product(&mut client),
            "4" => update_product(&mut client),
            "5" => delete_product(&mut client),
            "0" => {
                println!("\nExiting warehouse...\n");
                std::thread::sleep(std::time::Duration::from_secs(1));
                break;
            }
            _ => println!("Invalid option!"),
        }
    }
}
