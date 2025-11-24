mod api;
mod db;
type User = db::User;

fn main() {
    db::init_db();
    api::start_server();
}
