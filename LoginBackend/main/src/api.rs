use db;

type User = db::User;

const RANKS: Vec<&str> = vec!["Guest", "Archeologist", "Admin"];

#[get("/")]
fn index(user: User) -> String {
    format!("Hello, {}!", user.name);
}

#[post("/login", data = "<user>")]
fn login(user: User) -> String {
    let (is_valid, user) = db::search_user(&user.username, &user.password);
    if is_valid {
        format!("Welcome back, {}!", RANKS[user.rank]);
        if user.rank > 0 {
            format!("You are a member of the {}!", RANKS[user.rank])
        }
    } else {
        "Invalid username or password".to_string()
    }
    //TODO: store login in Cookies
}

#[post("/logout")]
fn logout(user: User) -> String {
    format!("Goodbye, {}!", user.name)
}
