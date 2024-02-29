use leptos::*;

#[derive(
    Debug, Clone, leptos::server_fn::serde::Serialize, leptos::server_fn::serde::Deserialize,
)]
pub struct Folder {
    id: String,
    parent_id: Option<String>,
    name: String,
}

#[server(TestDB, "/api")]
pub async fn test_db(name: String) -> Result<Vec<Folder>, ServerFnError> {
    use uuid::Uuid;
    use crate::components::db::db;

    // Connect to db
    let conn = db().await?;

    // Insert & parameters example. Uncomment to add to DB.
    use rusqlite::params;
    let _ = conn.execute("INSERT INTO folder (id, name, createdDate) values (?1, ?2, ?3)", 
        params![Uuid::new_v4().to_string(),name, "now:)"])?;

    let mut stmnt = conn.prepare("SELECT id, parentId, name FROM folder")?;

    let folders = stmnt.query_map([], |row| {
        Ok(Folder {
            id: row.get(0)?,
            parent_id: row.get(1)?,
            name: row.get(2)?,
        })
    })?;

    let mut vec = Vec::new();
    for folder in folders {
        vec.push(folder.unwrap());
    }

    use std::cmp;
    Ok(vec[cmp::max(vec.len() - cmp::min(vec.len(), 10), 0)..].to_vec())
}

#[component]
pub fn TestDBButton() -> impl IntoView {
    let (name, _set_name) = create_signal("Controlled".to_string());
    let input_el: NodeRef<html::Input> = create_node_ref();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let value = input_el().expect("<Input> should be mounted").value();
        spawn_local(async {
            logging::log!("{:?}", test_db(value).await.unwrap());
        });
    };

    view! {
        <form on:submit=on_submit>
            <input type="text"
                value=name
                node_ref=input_el
            />
            <input type="submit" value="Submit"/>
        </form>
    }
}
