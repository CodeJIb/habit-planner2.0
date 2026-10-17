use sqlx::SqlitePool;

use crate::errors::AppError;
use crate::models::category::{Category, CategoryInput};
use crate::repositories::category as repo;

fn is_hex_color(value: &str) -> bool {
    value.len() == 7 && value.starts_with('#') && value[1..].chars().all(|c| c.is_ascii_hexdigit())
}

pub struct CategoryService {
    pool: SqlitePool,
}

impl CategoryService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<Category>, AppError> {
        let mut conn = self.pool.acquire().await?;
        repo::list(&mut conn).await
    }

    pub async fn create(&self, mut input: CategoryInput) -> Result<Category, AppError> {
        input.name = input.name.trim().to_string();
        if input.name.is_empty() {
            return Err(AppError::InvalidInput("Category name is required".into()));
        }
        if !is_hex_color(&input.color) {
            return Err(AppError::InvalidInput("Color must look like #06b6d4".into()));
        }
        let mut conn = self.pool.acquire().await?;
        repo::insert(&mut conn, &input).await
    }

    pub async fn delete(&self, id: i64) -> Result<(), AppError> {
        let mut conn = self.pool.acquire().await?;
        if repo::delete(&mut conn, id).await? {
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_colors() {
        assert!(is_hex_color("#06b6d4"));
        assert!(!is_hex_color("06b6d4"));
        assert!(!is_hex_color("#06b6d"));
        assert!(!is_hex_color("#gggggg"));
    }
}
