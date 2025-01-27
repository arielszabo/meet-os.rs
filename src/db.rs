#![allow(clippy::std_instead_of_core)]

use chrono::{DateTime, Utc};

use rocket::fairing::AdHoc;
use surrealdb::engine::remote::ws::Client;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::Resource;
use surrealdb::Surreal;

use crate::{Audit, Counter, Event, Group, Membership, MyConfig, User};

/// # Panics
///
/// Panics when it fails to create the database folder or set up the database.
#[must_use]
pub fn fairing() -> AdHoc {
    // TODO handle errors here properly by using AdHoc::try_on_ignite instead of AdHoc::on_ignite.
    AdHoc::on_ignite("Managed Database Connection", |rocket| async {
        let config = rocket.state::<MyConfig>().unwrap();

        let dbh = get_database(&config.database_name, &config.database_namespace).await;
        rocket.manage(dbh)
    })
}

/// # Panics
///
/// Panics when it fails to create the database folder or set up the database.
pub async fn get_database(db_name: &str, db_namespace: &str) -> Surreal<Client> {
    let address = "127.0.0.1:8000";
    let dbh = Surreal::new::<Ws>(address).await.unwrap();
    dbh.use_ns(db_namespace).use_db(db_name).await.unwrap();
    // TODO: do this only when we create the database
    dbh.query("DEFINE INDEX user_email ON TABLE user COLUMNS email UNIQUE")
        .await
        .unwrap()
        .check()
        .unwrap();

    dbh.query("DEFINE INDEX member_ship ON TABLE membership COLUMNS uid, gid UNIQUE")
        .await
        .unwrap()
        .check()
        .unwrap();
    dbh
}

pub async fn add_user(dbh: &Surreal<Client>, user: &User) -> surrealdb::Result<()> {
    rocket::info!("add user email: '{}' code: '{}'", user.email, user.code);

    dbh.create(Resource::from("user")).content(user).await?;

    Ok(())
}

pub async fn add_event(dbh: &Surreal<Client>, event: &Event) -> surrealdb::Result<()> {
    rocket::info!("add event: '{}' code: '{}'", event.eid, event.title);

    dbh.create(Resource::from("event")).content(event).await?;

    Ok(())
}

pub async fn add_group(dbh: &Surreal<Client>, group: &Group) -> surrealdb::Result<()> {
    rocket::info!("add group: '{}'", group.name);

    dbh.create(Resource::from("group")).content(group).await?;

    Ok(())
}

pub async fn verify_code(
    dbh: &Surreal<Client>,
    process: &str,
    code: &str,
) -> surrealdb::Result<Option<User>> {
    rocket::info!("verification code: '{code}' process = '{process}'");
    let verified = true;

    let utc: DateTime<Utc> = Utc::now();
    let mut response = dbh
        .query(
            "
            UPDATE ONLY user
                SET
                    verified=$verified,
                    code='',
                    verification_date=$date
                WHERE code=$code AND process=$process;",
        )
        .bind(("verified", verified))
        .bind(("code", code))
        .bind(("date", utc))
        .bind(("process", process))
        .await?;

    let entry: Option<User> = response.take(0)?;

    if let Some(entry) = entry.as_ref() {
        rocket::info!(
            "verification ok '{}', '{}', '{}'",
            entry.name,
            entry.email,
            entry.process
        );
    }

    Ok(entry)
}

pub async fn update_group(
    dbh: &Surreal<Client>,
    gid: usize,
    name: &str,
    location: &str,
    description: &str,
) -> surrealdb::Result<Option<Group>> {
    rocket::info!("update group: '{gid}'");

    let mut response = dbh
        .query(
            "
            UPDATE group
            SET
                name=$name,
                location=$location,
                description=$description
            WHERE gid=$gid;",
        )
        .bind(("name", name))
        .bind(("location", location))
        .bind(("description", description))
        .bind(("gid", gid))
        .await?;

    let entry: Option<Group> = response.take(0)?;
    Ok(entry)
}

pub async fn update_user(
    dbh: &Surreal<Client>,
    uid: usize,
    name: &str,
    github: &str,
    gitlab: &str,
    linkedin: &str,
    about: &str,
) -> surrealdb::Result<Option<User>> {
    rocket::info!("update user: '{uid}'");

    let mut response = dbh
        .query(
            "
            UPDATE user
            SET
                name=$name,
                about=$about,
                gitlab=$gitlab,
                linkedin=$linkedin,
                github=$github
            WHERE uid=$uid;",
        )
        .bind(("name", name))
        .bind(("github", github))
        .bind(("gitlab", gitlab))
        .bind(("linkedin", linkedin))
        .bind(("uid", uid))
        .bind(("about", about))
        .await?;

    let entry: Option<User> = response.take(0)?;
    Ok(entry)
}

pub async fn get_user_by_id(dbh: &Surreal<Client>, uid: usize) -> surrealdb::Result<Option<User>> {
    rocket::info!("get_user_by_id: '{uid}'");

    let mut response = dbh
        .query("SELECT * FROM user WHERE uid=$uid;")
        .bind(("uid", uid))
        .await?;

    let entry: Option<User> = response.take(0)?;

    if let Some(entry) = entry.as_ref() {
        rocket::info!("Foud user {}, {}", entry.name, entry.email);
    }

    Ok(entry)
}

pub async fn get_user_by_email(
    dbh: &Surreal<Client>,
    email: &str,
) -> surrealdb::Result<Option<User>> {
    rocket::info!("get_user_by_email: '{email}'");
    rocket::info!("has db");
    let mut response = dbh
        .query("SELECT * FROM user WHERE email=$email;")
        .bind(("email", email))
        .await?;

    let entry: Option<User> = response.take(0)?;

    if let Some(entry) = entry.as_ref() {
        rocket::info!("************* {}, {}", entry.name, entry.email);
    }

    Ok(entry)
}

pub async fn add_login_code_to_user(
    dbh: &Surreal<Client>,
    email: &str,
    process: &str,
    code: &str,
) -> surrealdb::Result<Option<User>> {
    rocket::info!("add_login_code_to_user: '{email}', '{process}', '{code}'");

    rocket::info!("has db");
    let mut response = dbh
        .query("UPDATE user SET code=$code, process=$process WHERE email=$email;")
        .bind(("email", email))
        .bind(("process", process))
        .bind(("code", code))
        .await?;

    let entry: Option<User> = response.take(0)?;

    if let Some(entry) = entry.as_ref() {
        rocket::info!("entry: '{}' '{}'", entry.email, entry.process);
    }

    Ok(entry)
}

#[must_use]
pub async fn get_events_by_group_id(dbh: &Surreal<Client>, gid: usize) -> Vec<Event> {
    match get_events(dbh).await {
        Ok(events) => events
            .into_iter()
            .filter(|event| event.group_id == gid)
            .collect(),
        Err(_) => vec![],
    }
}

pub async fn get_users(dbh: &Surreal<Client>) -> surrealdb::Result<Vec<User>> {
    rocket::info!("get_users");
    let mut response = dbh.query("SELECT * FROM user;").await?;
    let entries: Vec<User> = response.take(0)?;
    for ent in &entries {
        rocket::info!("user name {}", ent.name);
    }
    Ok(entries)
}

pub async fn get_groups(dbh: &Surreal<Client>) -> surrealdb::Result<Vec<Group>> {
    rocket::info!("get_groups");
    let mut response = dbh.query("SELECT * FROM group;").await?;
    let entries: Vec<Group> = response.take(0)?;
    for ent in &entries {
        rocket::info!("group name {}", ent.name);
    }
    Ok(entries)
}

/// # Panics
///
/// Panics when there is an error
pub async fn get_groups_by_membership_id(
    dbh: &Surreal<Client>,
    uid: usize,
) -> surrealdb::Result<Vec<(Group, Membership)>> {
    rocket::info!("get_groups_by_membership_id: '{uid}'");

    // let mut response = dbh
    // .query("SELECT * FROM membership WHERE uid=$uid;")
    // .bind(("uid", uid))
    // .await?;

    // let entries: Vec<Membership> = response.take(0)?;
    // rocket::info!("gids: {entries:?}");

    // let mut response = dbh
    // .query("SELECT gid FROM membership WHERE uid=$uid;")
    // .bind(("uid", uid))
    // .await?;

    // let entries: Vec<String> = response.take(0)?;
    // rocket::info!("gids: {entries:?}");

    // let mut response = dbh
    //     .query("SELECT * FROM group WHERE gid IN (SELECT gid FROM membership WHERE uid=$uid);")
    //     .bind(("uid", uid))
    //     .await?;

    // let mut response = dbh
    //     .query("SELECT * FROM group, membership WHERE group.gid=membership.gid AND membership.uid=$uid;")
    //     .bind(("uid", uid))
    //     .await?;

    // TODO: make the code above with subexpression work
    let mut response = dbh
        .query("SELECT * FROM membership WHERE uid=$uid;")
        .bind(("uid", uid))
        .await?;

    let memberships: Vec<Membership> = response.take(0)?;
    rocket::info!("members: {memberships:?}");

    let mut groups = vec![];
    for member in memberships {
        rocket::info!("gid: {}", member.gid);
        let mut response2 = dbh
            .query("SELECT * FROM group WHERE gid=$gid;")
            .bind(("gid", member.gid))
            .await?;

        let entries: Vec<Group> = response2.take(0)?;
        rocket::info!("entries: {entries:?}");
        let group = entries.first().unwrap().clone();
        //groups.push((group, member.join_date));
        groups.push((group, member));
    }

    Ok(groups)
}

/// # Panics
///
/// Panics when there is an error
pub async fn get_members_of_group(
    dbh: &Surreal<Client>,
    gid: usize,
) -> surrealdb::Result<Vec<(User, Membership)>> {
    rocket::info!("get_members_of_group: '{gid}'");

    let mut response = dbh
        .query("SELECT * FROM membership WHERE gid=$gid;")
        .bind(("gid", gid))
        .await?;

    let memberships: Vec<Membership> = response.take(0)?;
    rocket::info!("members: {memberships:?}");

    let mut users = vec![];
    for member in memberships {
        rocket::info!("uid: {}", member.uid);
        let mut response2 = dbh
            .query("SELECT * FROM user WHERE uid=$uid;")
            .bind(("uid", member.uid))
            .await?;

        let entries: Vec<User> = response2.take(0)?;
        rocket::info!("entries: {entries:?}");
        let user = entries.first().unwrap().clone();
        users.push((user, member));
    }

    Ok(users)
}

pub async fn get_groups_by_owner_id(
    dbh: &Surreal<Client>,
    uid: usize,
) -> surrealdb::Result<Vec<Group>> {
    rocket::info!("get_groups_by_owner_id: '{uid}'");
    let mut response = dbh
        .query("SELECT * FROM group WHERE owner=$uid;")
        .bind(("uid", uid))
        .await?;

    let entries: Vec<Group> = response.take(0)?;

    Ok(entries)
}

pub async fn get_group_by_gid(
    dbh: &Surreal<Client>,
    gid: usize,
) -> surrealdb::Result<Option<Group>> {
    rocket::info!("get_group_by_gid: '{gid}'");
    let mut response = dbh
        .query("SELECT * FROM group WHERE gid=$gid;")
        .bind(("gid", gid))
        .await?;

    let entry: Option<Group> = response.take(0)?;

    if let Some(entry) = entry.as_ref() {
        rocket::info!("Group name: {}", entry.name);
    }

    Ok(entry)
}

pub async fn get_events(dbh: &Surreal<Client>) -> surrealdb::Result<Vec<Event>> {
    rocket::info!("get_events");
    let mut response = dbh.query("SELECT * FROM event;").await?;
    let entries: Vec<Event> = response.take(0)?;
    for ent in &entries {
        rocket::info!("event name {}", ent.title);
    }
    Ok(entries)
}

/// # Panics
///
/// Panics when there is an error
pub async fn increment(dbh: &Surreal<Client>, name: &str) -> surrealdb::Result<usize> {
    // TODO: do this only when creatig the database
    let _response = dbh
        .query("DEFINE INDEX counter_name ON TABLE counter COLUMNS name UNIQUE")
        .await?;

    #[allow(clippy::separated_literal_suffix)]
    let response = dbh
        .query(
            "
            INSERT INTO counter (name, count)
                VALUES ($name, $count) ON DUPLICATE KEY UPDATE count += 1;
        ",
        )
        .bind(("name", name))
        .bind(("count", 1_i32))
        .await?;

    let mut entries = response.check()?;
    let entries: Vec<Counter> = entries.take(0)?;
    // fetching the first (and hopefully only) entry
    let entry = entries.into_iter().next().unwrap();
    let id: usize = entry.count.try_into().unwrap();

    Ok(id)
}

pub async fn get_event_by_eid(
    dbh: &Surreal<Client>,
    eid: usize,
) -> surrealdb::Result<Option<Event>> {
    rocket::info!("get_event_by_eid: '{eid}'");
    let mut response = dbh
        .query("SELECT * FROM event WHERE eid=$eid;")
        .bind(("eid", eid))
        .await?;

    let entry: Option<Event> = response.take(0)?;

    if let Some(entry) = entry.as_ref() {
        rocket::info!("Event title: {}", entry.title);
    }

    Ok(entry)
}

pub async fn join_group(dbh: &Surreal<Client>, gid: usize, uid: usize) -> surrealdb::Result<()> {
    rocket::info!("user {} joins group: {}", uid, gid);

    let date: DateTime<Utc> = Utc::now();

    let membership = Membership {
        uid,
        gid,
        join_date: date,
        admin: false,
    };

    dbh.create(Resource::from("membership"))
        .content(membership)
        .await?;

    Ok(())
}

/// # Panics
///
/// Panics when it fails
pub async fn leave_group(dbh: &Surreal<Client>, gid: usize, uid: usize) -> surrealdb::Result<()> {
    rocket::info!("user {} leaves group: {}", uid, gid);

    dbh.query("DELETE membership WHERE uid=$uid AND gid=$gid")
        .bind(("uid", uid))
        .bind(("gid", gid))
        .await?
        .check()
        .unwrap();

    Ok(())
}

pub async fn get_membership(
    dbh: &Surreal<Client>,
    gid: usize,
    uid: usize,
) -> surrealdb::Result<Option<Membership>> {
    let mut response = dbh
        .query("SELECT * FROM membership WHERE gid=$gid AND uid=$uid;")
        .bind(("gid", gid))
        .bind(("uid", uid))
        .await?;

    let entry: Option<Membership> = response.take(0)?;

    Ok(entry)
}

pub async fn audit(dbh: &Surreal<Client>, text: String) -> surrealdb::Result<()> {
    rocket::info!("audit {text}");

    let date: DateTime<Utc> = Utc::now();
    let entry = Audit { date, text };

    dbh.create(Resource::from("audit")).content(entry).await?;

    Ok(())
}

pub async fn get_audit(dbh: &Surreal<Client>) -> surrealdb::Result<Vec<Audit>> {
    rocket::info!("get_audits");
    let mut response = dbh.query("SELECT * FROM audit;").await?;
    let entries: Vec<Audit> = response.take(0)?;
    Ok(entries)
}
