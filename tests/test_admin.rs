//use regex::Regex;

use utilities::{login_helper, register_user_helper, run_external};

#[test]
fn admin_list_users() {
    run_external(|port, email_folder| {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://localhost:{port}/");
        let _cookie_str = register_user_helper(
            &client,
            &url,
            "Foo Bar",
            "foo@meet-os.com",
            "123foo",
            &email_folder,
        );

        let name = "Site Manager";
        let email = "admin@meet-os.com";
        let password = "123456";

        let admin_cookie_str =
            register_user_helper(&client, &url, name, email, password, &email_folder);
        login_helper(&client, &url, email, password);

        let res = client
            .get(format!("{url}/users"))
            .header("Cookie", format!("meet-os={admin_cookie_str}"))
            .send()
            .unwrap();

        assert_eq!(res.status(), 200);
        let html = res.text().unwrap();
        //println!("{html}");
        assert!(html.contains("Foo Bar"));
        assert!(html.contains(name));
    });
}
