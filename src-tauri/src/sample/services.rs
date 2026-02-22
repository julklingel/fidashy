
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
            .map_err(|e| e.to_string())?;

        db.execute(
            "INSERT INTO greeted_people (id, name) VALUES (?, ?)",
            params![next_id, name],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    })
}

pub fn list_greeted_people(db_state: &DuckDbState) -> Result<Vec<String>, String> {
    db_state.with_db(|db| {
        let mut sql_stmt = db.prepare("SELECT name FROM greeted_people ORDER BY greeted_at DESC")
            .map_err(|e| e.to_string())?;

        let names = sql_stmt
            .query_map([], |row| row.get::<_, String>(0)) 
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>() 
            .map_err(|e| e.to_string())?;

        Ok(names)
    })
}