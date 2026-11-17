
# Mosconi Warehouse

Mosconi Warehouse is a simple inventory management application built with Rust, PostgreSQL, HTML, CSS, and JavaScript.

The project demonstrates backend development with Rust, REST API implementation, database integration, automated testing, and frontend development.

If you're testing Mosconi Warehouse, it would be fun to add products inspired by Germano Mosconi, such as **Porcodrillo**, **Mona che sbatte e chiude la porta urlando**, or **Portanna la Madonna**. Of course, you're free to add any products you like!

## Features

- View all products in the inventory
- View individual products through the REST API
- Add new products
- Edit existing products
- Delete products with confirmation
- Cancel product editing
- Validate product data
- Responsive web interface
- Store product information in PostgreSQL
- Automated API and database tests

## Screenshots

### Inventory Management

![Inventory Management](Images/inventory.png) 

### Edit Product

![Edit product](Images/edit-product.png)

## Technologies

### Backend
- Rust
- Axum
- Tokio
- tokio-postgres
- Serde
- dotenvy

### Frontend
- HTML5
- CSS3
- JavaScript

### Database
- PostgreSQL

### Testing
- Rust built-in testing framework
- Tower
- http-body-util

## Project Structure

rust-inventory/
├── database/
│   └── schema.sql
├── Images/
│   ├── inventory.png
│   └── edit-product.png
├── src/
│   ├── bin/
│   │   └── api.rs
│   └── main.rs
├── static/
│   ├── app.js
│   ├── index.html
│   └── style.css
├── .gitignore
├── Cargo.lock
├── Cargo.toml
└── README.md

## Requirements

Before running the application, install:

- Rust and Cargo
- PostgreSQL

## Database Setup

1. Create a PostgreSQL database named `mosconi_warehouse`.

2. Run the SQL script located at `database/schema.sql` to create the `products` table.

3. Create a `.env` file in the project root directory.

4. Add your PostgreSQL connection string:

```env
DATABASE_URL=postgresql://username:password@localhost:5432/mosconi_warehouse
```

Replace `username` and `password` with your PostgreSQL credentials.

The `.env` file is excluded from Git to protect database credentials.

## Running the Application

Open a terminal in the project directory and run:

```bash
cargo run --bin api
```

The application will be available at:

http://127.0.0.1:3000

Open this address in your browser to use the inventory management interface.

The project also includes a command-line application:

```bash
cargo run --bin rust-inventory
```

## REST API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/products` | Get all products |
| GET | `/products/{id}` | Get a product by ID |
| POST | `/products` | Create a new product |
| PUT | `/products/{id}` | Update an existing product |
| DELETE | `/products/{id}` | Delete a product |

### Example JSON Request

```json
{
  "name": "Porcodrillo",
  "quantity": 25,
  "price": 49.99
}
```

## Automated Tests

The project includes 10 automated tests covering:

- Product serialization
- API route handling
- Product ID validation
- Test database connection
- Retrieving products
- Handling nonexistent products
- Creating products
- Updating products
- Deleting products
- Rejecting invalid product quantities

### Test Database Setup

Create a separate PostgreSQL database named `mosconi_warehouse_test`.

Run `database/schema.sql` in the test database.

Create a `.env.test` file in the project root:

```env
DATABASE_URL=postgresql://username:password@localhost:5432/mosconi_warehouse_test
```

The test database is separate from the main database to prevent automated tests from modifying production data.

### Run Tests

```bash
cargo test --bin api
```

## Project Purpose

Mosconi Warehouse was developed as a personal portfolio project to practice Rust backend programming, REST API development, PostgreSQL integration, automated testing, and frontend development.

The project combines a command-line application with a browser-based inventory management interface.

## Author

Daniel Pozzoli
```
