use diesel::PgConnection;
#[database("postgres_database")]
pub struct DbConn(PgConnection);

fn main() {
    print!("{}", get_env("DATABASE_URL".to_string()));
    rocket().0.launch();
}
