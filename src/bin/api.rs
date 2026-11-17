use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres::Client;
use tokio_postgres::NoTls;
use tower_http::services::{ServeDir, ServeFile};

// Shared database connection
type DbState = Arc<Mutex<Client>>;

// Product data returned as JSON
#[derive(Serialize)]
struct Product {
    id: i32,
    name: String,
    quantity: i32,
    price: f64,
}

// Validate a product ID
fn validate_product_id(id: i32) -> Result<(), StatusCode> {
    if id <= 0 {
        Err(StatusCode::BAD_REQUEST)
    } else {
        Ok(())
    }
}

// Product data received from a POST request
#[derive(Deserialize)]
struct NewProduct {
    name: String,
    quantity: i32,
    price: f64,
}

// Product data received from a PUT request
#[derive(Deserialize)]
struct UpdateProduct {
    name: String,
    quantity: i32,
    price: f64,
}

// Retrieve all products from PostgreSQL
async fn get_products(State(db): State<DbState>) -> Result<Json<Vec<Product>>, StatusCode> {
    // Access the shared database connection
    let client = db.lock().await;

    // Fetch products ordered by ID
    let rows = client
        .query(
            "SELECT id, name, quantity, price::float8 FROM products ORDER BY id",
            &[],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert database rows into Product structures
    let products: Vec<Product> = rows
        .iter()
        .map(|row| Product {
            id: row.get(0),
            name: row.get(1),
            quantity: row.get(2),
            price: row.get(3),
        })
        .collect();

    Ok(Json(products))
}

// Create a new product in PostgreSQL
async fn create_product(
    State(db): State<DbState>,
    Json(product): Json<NewProduct>,
) -> Result<(StatusCode, Json<Product>), StatusCode> {
    // Validate product data
    if product.name.trim().is_empty()
        || product.name.chars().count() > 255
        || product.quantity < 0
        || !product.price.is_finite()
        || product.price < 0.0
        || product.price > 99_999_999.99
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Access the shared database connection
    let client = db.lock().await;

    // Insert the product and return its generated ID
    let row = client
        .query_one(
            "INSERT INTO products (name, quantity, price)
             VALUES ($1, $2, $3::float8::numeric)
             RETURNING id, name, quantity, price::float8",
            &[&product.name.trim(), &product.quantity, &product.price],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert the inserted row into a Product
    let created = Product {
        id: row.get(0),
        name: row.get(1),
        quantity: row.get(2),
        price: row.get(3),
    };

    // HTTP 201 Created
    Ok((StatusCode::CREATED, Json(created)))
}

// Update an existing product in PostgreSQL
async fn update_product(
    State(db): State<DbState>,
    Path(id): Path<i32>,
    Json(product): Json<UpdateProduct>,
) -> Result<Json<Product>, StatusCode> {
    // Validate the product ID
    validate_product_id(id)?;

    // Validate the updated product data
    if product.name.trim().is_empty()
        || product.name.chars().count() > 255
        || product.quantity < 0
        || !product.price.is_finite()
        || product.price < 0.0
        || product.price > 99_999_999.99
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Access the shared database connection
    let client = db.lock().await;

    // Update the product and retrieve its new values
    let row = client
        .query_opt(
            "UPDATE products
             SET name = $1, quantity = $2, price = $3::float8::numeric
             WHERE id = $4
             RETURNING id, name, quantity, price::float8",
            &[&product.name.trim(), &product.quantity, &product.price, &id],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Return the updated product or HTTP 404
    match row {
        Some(row) => Ok(Json(Product {
            id: row.get(0),
            name: row.get(1),
            quantity: row.get(2),
            price: row.get(3),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// Delete a product by ID
async fn delete_product(
    State(db): State<DbState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    // Validate the product ID
    validate_product_id(id)?;

    // Access the shared database connection
    let client = db.lock().await;

    // Delete the product from PostgreSQL
    let deleted = client
        .execute("DELETE FROM products WHERE id = $1", &[&id])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Return HTTP 204 if deleted, or HTTP 404 if not found
    if deleted > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Retrieve a single product by ID
async fn get_product(
    State(db): State<DbState>,
    Path(id): Path<i32>,
) -> Result<Json<Product>, StatusCode> {
    // Reject invalid IDs
    validate_product_id(id)?;

    // Access the shared database connection
    let client = db.lock().await;

    // Search for the requested product
    let row = client
        .query_opt(
            "SELECT id, name, quantity, price::float8 FROM products WHERE id = $1",
            &[&id],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Return the product or HTTP 404 if it does not exist
    match row {
        Some(row) => Ok(Json(Product {
            id: row.get(0),
            name: row.get(1),
            quantity: row.get(2),
            price: row.get(3),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().expect("Failed to load .env file");

    // Read database connection settings
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not found");

    // Connect to PostgreSQL asynchronously
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Keep the database connection running in the background
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!("PostgreSQL connection error: {}", error);
        }
    });

    // Verify the database connection
    let row = client
        .query_one("SELECT COUNT(*) FROM products", &[])
        .await
        .expect("Failed to count products");

    let count: i64 = row.get(0);

    println!("Connected to PostgreSQL successfully!");
    println!("Products in database: {}", count);
    // Share the database client with API handlers
    let db: DbState = Arc::new(Mutex::new(client));

    // Define the API routes
    let app = Router::new()
        .route("/products", get(get_products).post(create_product))
        .route(
            "/products/{id}",
            get(get_product).put(update_product).delete(delete_product),
        )
        .fallback_service(
            ServeDir::new("static").not_found_service(ServeFile::new("static/index.html")),
        )
        .with_state(db);

    // Start the HTTP server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to start server");

    println!("Mosconi Warehouse API running at http://127.0.0.1:3000");

    axum::serve(listener, app).await.expect("Server error");
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_db() -> DbState {
        // Read the test configuration without changing environment variables
        let config = std::fs::read_to_string(".env.test").expect("Cannot read .env.test");

        let database_url = config
            .lines()
            .find_map(|line| line.trim().strip_prefix("DATABASE_URL="))
            .expect("DATABASE_URL not found in .env.test")
            .trim()
            .trim_matches('"');

        // Prevent tests from connecting to the production database
        let pg_config: tokio_postgres::Config =
            database_url.parse().expect("Invalid test database URL");

        assert_eq!(
            pg_config.get_dbname(),
            Some("mosconi_warehouse_test"),
            "Tests must use mosconi_warehouse_test"
        );

        // Connect only to the test database
        let (client, connection) = pg_config
            .connect(NoTls)
            .await
            .expect("Cannot connect to test database");

        tokio::spawn(async move {
            if let Err(error) = connection.await {
                eprintln!("Test database connection error: {error}");
            }
        });

        Arc::new(Mutex::new(client))
    }

    #[test]
    fn test_product_serialization() {
        let product = Product {
            id: 2,
            name: "Gaetano".to_string(),
            quantity: 25,
            price: 10.99,
        };

        assert_eq!(product.id, 2);
        assert_eq!(product.name, "Gaetano");
        assert_eq!(product.quantity, 25);
        assert_eq!(product.price, 10.99);

        // Serialize the product into JSON
        let json = serde_json::to_value(&product).unwrap();

        // Verify the JSON fields
        assert_eq!(json["id"], 2);
        assert_eq!(json["name"], "Gaetano");
        assert_eq!(json["quantity"], 25);
        assert_eq!(json["price"], 10.99);
    }

    #[tokio::test]
    async fn test_database_connection() {
        let db = create_test_db().await;
        let client = db.lock().await;

        // Verify the test database connection
        let row = client
            .query_one("SELECT current_database()", &[])
            .await
            .unwrap();

        let database_name: String = row.get(0);
        assert_eq!(database_name, "mosconi_warehouse_test");
    }

    #[tokio::test]
    async fn test_unknown_route_returns_404() {
        use axum::body::Body;
        use axum::http::{Request, StatusCode};
        use tower::ServiceExt;

        // Create a test router without database access
        let app = Router::new();

        // Send a GET request to an unknown endpoint
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/unknown")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Verify the HTTP status code
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_product_id_validation() {
        // Valid product IDs
        assert_eq!(validate_product_id(1), Ok(()));
        assert_eq!(validate_product_id(2), Ok(()));

        // Invalid product IDs
        assert_eq!(validate_product_id(0), Err(StatusCode::BAD_REQUEST));
        assert_eq!(validate_product_id(-5), Err(StatusCode::BAD_REQUEST));
    }

    #[tokio::test]
    async fn test_get_products_returns_200() {
        use axum::body::Body;
        use axum::http::Request;
        use http_body_util::BodyExt;
        use tower::ServiceExt;

        // Connect to the separate test database
        let db = create_test_db().await;

        // Create a router with the real GET handler
        let app = Router::new()
            .route("/products", get(get_products))
            .with_state(db);

        // Send an HTTP request without starting the server
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/products")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Verify the HTTP response
        assert_eq!(response.status(), StatusCode::OK);
        // Read the HTTP response body
        let body = response.into_body().collect().await.unwrap().to_bytes();

        // Parse the response as JSON
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Verify that the API returns an array
        assert!(json.is_array());
    }

    #[tokio::test]
    async fn test_get_nonexistent_product_returns_404() {
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;

        // Connect to the test database
        let db = create_test_db().await;

        // Create a router with the real product handler
        let app = Router::new()
            .route("/products/{id}", get(get_product))
            .with_state(db);

        // Request a product that does not exist
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/products/999999999")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Verify the HTTP status code
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_and_get_product() {
        use axum::body::Body;
        use axum::http::{Method, Request};
        use http_body_util::BodyExt;
        use tower::ServiceExt;

        // Connect to the test database
        let db = create_test_db().await;

        let app = Router::new()
            .route("/products", get(get_products).post(create_product))
            .route("/products/{id}", get(get_product))
            .with_state(db.clone());

        // Create a test product
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/products")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"name":"Test Product","quantity":10,"price":19.99}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let product: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = product["id"].as_i64().unwrap();

        // Retrieve the newly created product
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/products/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let retrieved: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(retrieved["id"], id);
        assert_eq!(retrieved["name"], "Test Product");
        assert_eq!(retrieved["quantity"], 10);
        assert_eq!(retrieved["price"], 19.99);

        // Remove the test product
        let client = db.lock().await;
        client
            .execute("DELETE FROM products WHERE id = $1", &[&(id as i32)])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_update_product() {
        use axum::body::Body;
        use axum::http::{Method, Request};
        use http_body_util::BodyExt;
        use tower::ServiceExt;

        // Connect to the test database
        let db = create_test_db().await;

        let app = Router::new()
            .route("/products", get(get_products).post(create_product))
            .route("/products/{id}", get(get_product).put(update_product))
            .with_state(db.clone());

        // Create a test product
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/products")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"name":"Test Product","quantity":10,"price":19.99}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let product: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = product["id"].as_i64().unwrap();

        // Update the product using PUT
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri(format!("/products/{id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"name":"Updated Product","quantity":50,"price":29.99}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify the updated JSON response
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(updated["id"], id);
        assert_eq!(updated["name"], "Updated Product");
        assert_eq!(updated["quantity"], 50);
        assert_eq!(updated["price"], 29.99);

        // Remove the test product
        let client = db.lock().await;
        client
            .execute("DELETE FROM products WHERE id = $1", &[&(id as i32)])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_delete_product() {
        use axum::body::Body;
        use axum::http::{Method, Request};
        use http_body_util::BodyExt;
        use tower::ServiceExt;

        // Connect to the test database
        let db = create_test_db().await;

        let app = Router::new()
            .route("/products", get(get_products).post(create_product))
            .route("/products/{id}", get(get_product).delete(delete_product))
            .with_state(db);

        // Create a test product
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/products")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"name":"Delete Test","quantity":5,"price":9.99}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let product: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = product["id"].as_i64().unwrap();

        // Delete the product through the API
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(format!("/products/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // Verify that the deleted product no longer exists
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/products/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_product_invalid_quantity() {
        use axum::body::Body;
        use axum::http::{Method, Request};
        use tower::ServiceExt;

        // Connect to the test database
        let db = create_test_db().await;

        let app = Router::new()
            .route("/products", get(get_products).post(create_product))
            .with_state(db);

        // Try to create a product with a negative quantity
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/products")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"name":"Invalid Product","quantity":-10,"price":19.99}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Verify that invalid data is rejected
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
