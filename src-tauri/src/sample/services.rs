
use crate::db::DuckDbState;
use duckdb::params;



pub fn save_greeting(db_state: &DuckDbState, name: &str) -> Result<(), String> {
    db_state.with_db(|db| {
        let next_id: i64 = db
            .query_row(
                "SELECT COALESCE(MAX(id), 0) + 1 FROM greeted_people",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to compute next greeting id: {e}"))?;

        db.execute(
            "INSERT INTO greeted_people (id, name) VALUES (?, ?)",
            params![next_id, name],
        )
        .map_err(|e| format!("Failed to save greeting: {e}"))?;

        Ok(())
    })
}

pub fn list_greeted_people(db_state: &DuckDbState) -> Result<Vec<String>, String> {
    db_state.with_db(|db| {
        let mut statement = db
            .prepare("SELECT name FROM greeted_people ORDER BY greeted_at DESC")
            .map_err(|e| format!("Failed to prepare greeted people query: {e}"))?;

        let rows = statement
            .query_map([], |row| row.get::<usize, String>(0))
            .map_err(|e| format!("Failed to query greeted people: {e}"))?;

        let mut names = Vec::new();
        for row in rows {
            names.push(row.map_err(|e| format!("Failed to read greeted person row: {e}"))?);
        }

        Ok(names)
    })
}