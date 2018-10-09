use super::config::ROOT_ROUTE;
use super::mount_paste;
use rocket::http::{ContentType, Status};
use rocket::local::Client;

fn rocket() -> rocket::Rocket {
    mount_paste(rocket::ignite())
}

fn extract_id(from: &str) -> Option<String> {
    from.rfind('/')
        .map(|i| &from[(i + 1)..])
        .map(|s| s.trim_right().to_string())
}

fn upload_paste(client: &Client, body: &str) -> String {
    let query = format!("content={}", body);
    let response = client
        .post(format!("{}/upload", ROOT_ROUTE))
        .header(ContentType::Form)
        .body(query)
        .dispatch();

    assert_eq!(response.status(), Status::SeeOther);
    let location_header = response.headers().get_one("Location").unwrap();
    extract_id(location_header).unwrap()
}

fn download_paste(client: &Client, id: &str) -> String {
    let mut response = client
        .get(format!("{}/view/raw/{}", ROOT_ROUTE, id))
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    response.body_string().unwrap()
}

#[test]
fn pasting() {
    let client = Client::new(rocket()).unwrap();

    // Do a trivial upload, just to make sure it works.
    let paste_1 = "A very simple paste.";
    let id_1 = upload_paste(&client, paste_1);
    assert_eq!(download_paste(&client, &id_1), paste_1);

    // Make sure we can keep getting that paste.
    assert_eq!(download_paste(&client, &id_1), paste_1);
    assert_eq!(download_paste(&client, &id_1), paste_1);
    assert_eq!(download_paste(&client, &id_1), paste_1);

    // Upload some unicode.
    let paste_2 = "こんにちは、花子です。";
    let id_2 = upload_paste(&client, paste_2);
    assert_eq!(download_paste(&client, &id_2), paste_2);

    // Make sure we can get both pastes.
    assert_eq!(download_paste(&client, &id_1), paste_1);
    assert_eq!(download_paste(&client, &id_2), paste_2);
    assert_eq!(download_paste(&client, &id_1), paste_1);
    assert_eq!(download_paste(&client, &id_2), paste_2);

    // Now a longer upload.
    let paste_3 = "Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed
        do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim
        ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut
        aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit
        in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
        Excepteur sint occaecat cupidatat non proident, sunt in culpa qui
        officia deserunt mollit anim id est laborum.";
    let id_3 = upload_paste(&client, paste_3);
    assert_eq!(download_paste(&client, &id_3), paste_3);
    assert_eq!(download_paste(&client, &id_1), paste_1);
    assert_eq!(download_paste(&client, &id_2), paste_2);
}
