use actix_web::{
    web, 
    App, 
    HttpServer, 
    Responder, 
    HttpResponse,
    middleware::Logger
};

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::env;
use std::sync::Arc;
use sqlx::migrate::MigrateDatabase;
use serde::{Serialize, Deserialize};

// Database connection and migration helper
pub struct DatabaseConnection {
    pub pool: SqlitePool,
}

impl DatabaseConnection {
    // Initialize database connection and run migrations
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        // Create database pool
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;
        Ok(DatabaseConnection { pool })
    }
}



// Application state with database connection
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<SqlitePool>,
}

// Main application setup function
pub async fn setup_database() -> Result<AppState, sqlx::Error> {
    // Load database URL from environment or use default
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:users.db".to_string());

    // Ensure database exists and is migrated
    if !sqlx::sqlite::Sqlite::database_exists(&database_url).await
        .unwrap_or(false) 
    {
        println!("Creating database {}", database_url);
        sqlx::sqlite::Sqlite::create_database(&database_url).await?;
    }

    // Initialize database connection
    let db_conn = DatabaseConnection::new(&database_url).await?;

    // Wrap pool in Arc for thread-safe sharing
    let app_state = AppState {
        db: Arc::new(db_conn.pool),
    };

    Ok(app_state)
}

// Example main function integrating database setup
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
   // Initialize logging
    env_logger::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Setup database
    let app_state = setup_database()
        .await
        .expect("Failed to setup database");

     // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(app_state.clone()))
            .service(
                web::scope("/users")
                    .route("", web::get().to(list_users))
                    .route("", web::post().to(create_user))
                    .route("/{id}", web::get().to(get_user))
                    .route("/{id}", web::delete().to(delete_user))
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
    
}

// Example repository pattern for database operations
pub struct UserRepository;

impl UserRepository {
    // List all users
    pub async fn list_users(pool: &SqlitePool) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as!(
            User, 
            "SELECT id, username, email FROM users"
        )
        .fetch_all(pool)
        .await
    }

    // Create a new user
    pub async fn create_user(
        pool: &SqlitePool, 
        username: &str, 
        email: &str
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            "INSERT INTO users (username, email) VALUES (?, ?) RETURNING id, username, email",
            username,
            email
        )
        .fetch_one(pool)
        .await
    }

    // Get user by ID
    pub async fn get_user_by_id(
        pool: &SqlitePool, 
        user_id: i64
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT id, username, email FROM users WHERE id = ?",
            user_id
        )
        .fetch_optional(pool)
        .await
    }

    // Delete user by ID
    pub async fn delete_user(
        pool: &SqlitePool, 
        user_id: i64
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "DELETE FROM users WHERE id = ? RETURNING id, username, email",
            user_id
        )
        .fetch_optional(pool)
        .await
    }
}



// Route Handlers
// List Users Handler
async fn list_users(data: web::Data<AppState>) -> impl Responder {
    match UserRepository::list_users(&data.db).await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(_) => HttpResponse::InternalServerError().body("Error fetching users"),
    }
}

// Create User Handler
async fn create_user(
    data: web::Data<AppState>, 
    user: web::Json<User>
) -> impl Responder {
    match UserRepository::create_user(
        &data.db, 
        &user.username, 
        &user.email
    ).await {
        Ok(created_user) => HttpResponse::Created().json(created_user),
        Err(_) => HttpResponse::InternalServerError().body("Error creating user"),
    }
}

// Get User Handler
async fn get_user(
    data: web::Data<AppState>, 
    path: web::Path<i64>
) -> impl Responder {
    let user_id = path.into_inner();
    
    match UserRepository::get_user_by_id(&data.db, user_id).await {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().body("User not found"),
        Err(_) => HttpResponse::InternalServerError().body("Error fetching user"),
    }
}

// Delete User Handler
async fn delete_user(
    data: web::Data<AppState>, 
    path: web::Path<i64>
) -> impl Responder {
    let user_id = path.into_inner();
    
    match UserRepository::delete_user(&data.db, user_id).await {
        Ok(Some(deleted_user)) => HttpResponse::Ok().json(deleted_user),
        Ok(None) => HttpResponse::NotFound().body("User not found"),
        Err(_) => HttpResponse::InternalServerError().body("Error deleting user"),
    }
}


// User Model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub email: String,
}

