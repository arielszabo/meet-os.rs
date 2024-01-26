//use regex::Regex;

use utilities::{check_html, register_user_helper, run_external};

#[test]
fn create_group() {
    run_external(|port| {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://localhost:{port}/");

        let foo_cookie_str = register_user_helper(&client, &url, "Foo Bar", "foo@meet-os.com");
        println!("foo_cookie_str: {foo_cookie_str}");
        let peti_cookie_str = register_user_helper(&client, &url, "Peti Bar", "peti@meet-os.com");
        println!("peti_cookie_str: {peti_cookie_str}");

        // Access the Group creation page with unauthorized user
        let res = client
            .get(format!("{url}/create-group"))
            .header("Cookie", format!("meet-os={foo_cookie_str}"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);

        let html = res.text().unwrap();
        //assert_eq!(html, "x");
        check_html(&html, "title", "Create Group");
        check_html(&html, "h1", "Create Group");
    });
}

#[test]
fn create_group_unauthorized() {
    run_external(|port| {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://localhost:{port}/");

        let peti_cookie_str = register_user_helper(&client, &url, "Peti Bar", "peti@meet-os.com");
        println!("peti_cookie_str: {peti_cookie_str}");

        // Access the Group creation page without user
        let res = client.get(format!("{url}/create-group")).send().unwrap();
        assert_eq!(res.status(), 200);

        let html = res.text().unwrap();
        //assert_eq!(html, "x");
        check_html(&html, "title", "Not logged in");
        check_html(&html, "h1", "Not logged in");

        // Access the Group creation page with unauthorized user
        let res = client
            .get(format!("{url}/create-group"))
            .header("Cookie", format!("meet-os={peti_cookie_str}"))
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);

        let html = res.text().unwrap();
        //assert_eq!(html, "x");
        check_html(&html, "title", "Unauthorized");
        check_html(&html, "h1", "Unauthorized");
    });
}
