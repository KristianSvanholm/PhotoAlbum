#[cfg(feature = "ssr")]
use crate::auth;
use crate::auth::User;
use crate::components::dialog::Dialog;
#[cfg(feature = "ssr")]
use crate::components::upload::Bbox;
use crate::components::upload::Person;
use crate::components::upload::{decode_image, img_from_bounds};
use crate::components::users::get_user_list_sans_admin;
use image::DynamicImage;
use leptonic::components::icon::Icon;
use leptonic::components::prelude::OptionalSelect;
use leptos::html::Input;
use leptos::*;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
#[cfg(feature = "ssr")]
use std::fs::File;
#[cfg(feature = "ssr")]
use std::io::Read;
use std::ops::Not;

//Image struct for images from DB
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct ImageDb {
    id: String,
    path: String,
    upload_date: String,
    created_date: Option<String>,
    uploader: String,
    location: Option<String>,
}
impl ImageDb {
    pub fn into_info(self) -> ImageInfo {
        ImageInfo {
            id: self.id,
            upload_date: self.upload_date,
            created_date: self.created_date,
            uploader: self.uploader,
            location: self.location,
        }
    }
}

#[derive(Clone)]
pub struct ImageInfo {
    id: String,
    upload_date: String,
    created_date: Option<String>,
    uploader: String,
    location: Option<String>,
}
impl Default for ImageInfo {
    fn default() -> Self {
        Self {
            id: "".into(),
            upload_date: "".into(),
            created_date: None,
            uploader: "".into(),
            location: None,
        }
    }
}

#[cfg(feature = "ssr")]
use sqlx::sqlite::SqliteRow;
#[cfg(feature = "ssr")]
use sqlx::{Error, FromRow, Row};
#[cfg(feature = "ssr")]
impl<'a> FromRow<'a, SqliteRow> for Person {
    fn from_row(row: &'a SqliteRow) -> Result<Self, Error> {
        let name: String = row.try_get("name")?;
        let id: i64 = row.try_get("id")?;
        let x: Option<u32> = row.try_get("x")?;
        let y: Option<u32> = row.try_get("y")?;
        let width: Option<u32> = row.try_get("w")?;
        let height: Option<u32> = row.try_get("h")?;
        let mut bounds = None;
        if x.is_some() && y.is_some() && width.is_some() && height.is_some() {
            bounds = Some(Bbox {
                x: x.unwrap(),
                y: y.unwrap(),
                w: width.unwrap(),
                h: height.unwrap(),
            });
        }
        ::std::result::Result::Ok(Person {
            name: name,
            id: id as i64,
            bounds: bounds,
        })
    }
}

//Fetch images from database
#[server(Image, "/api")]
pub async fn get_image(image_id: String) -> Result<ImageDb, ServerFnError> {
    auth::logged_in().await?;

    //DB connection
    use crate::app::ssr::*;
    let pool = pool()?;

    //Fetch image
    let mut img = sqlx::query_as::<_, ImageDb>(
        "SELECT files.id, path, uploadDate AS upload_date, createdDate AS created_date, users.username AS uploader, location 
        FROM files INNER JOIN users ON files.uploadedBy=users.id WHERE files.id = ?;",
    )
    .bind(image_id)
    .fetch_one(&pool)
    .await?;

    // Read the image file
    let mut file = File::open(&img.path).expect("Failed to open image file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .expect("Failed to read image file");

    // Encode the image buffer as base64
    let base64_image = base64::encode(&buffer);

    // Generate src attribute value with the base64 image
    img.path = base64_image;

    Ok(img)
}

//Update image info
#[server(UpdateImageInfo, "/api")]
pub async fn update_image_info(
    image_id: String,
    created_date: Option<String>,
    location: Option<String>,
) -> Result<(), ServerFnError> {
    let user = auth::logged_in().await?;

    //DB connection
    use crate::app::ssr::*;
    let pool = pool()?;

    //only uploader or admin
    let uploader: bool = sqlx::query_scalar("SELECT uploadedBy=? FROM files WHERE id = ?")
        .bind(user.id)
        .bind(image_id.clone())
        .fetch_one(&pool)
        .await?;

    if !uploader && !user.has("admin") {
        return Err(ServerFnError::ServerError(
            "You are not authorized, only the uploader can change an image".to_string(),
        ));
    }

    //Check if created_date is a valid date
    use regex::Regex;
    if let Some(date) = created_date.clone() {
        let valid_date = Regex::new(r"^\d{4}-(0[1-9]|1[012])-(0[1-9]|[12][0-9]|3[01])$")
            .unwrap()
            .is_match(&date);
        if !valid_date {
            return Err(ServerFnError::ServerError(
                "The date is corrupted. The date must have the format yyyy-mm-dd".to_string(),
            ));
        }
    }

    //Update image
    sqlx::query("UPDATE files SET createdDate=?,location=? WHERE id = ?;")
        .bind(created_date)
        .bind(location)
        .bind(image_id)
        .execute(&pool)
        .await?;

    Ok(())
}

//Fetch images from database
#[server(UseresInImage, "/api")]
pub async fn get_users_in_image(image_id: String) -> Result<Vec<Person>, ServerFnError> {
    auth::logged_in().await?;

    //DB connection
    use crate::app::ssr::*;
    let pool = pool()?;

    //Fetch users
    let users = sqlx::query_as::<_, Person>(
        "SELECT x, y, width AS w, height AS h, users.username AS name, users.id as id 
        FROM users 
        JOIN userFile 
        ON users.id = userFile.userID 
        WHERE userFile.fileID = ?;",
    )
    .bind(image_id)
    .fetch_all(&pool)
    .await?;

    Ok(users)
}

//Update image info
#[server(UpdateUseresInImage, "/api")]
pub async fn update_users_in_image(
    image_id: String,
    #[server(default)] users_change: Vec<(i64, Person)>,
    #[server(default)] users_delete: Vec<i64>,
    #[server(default)] users_add: Vec<Person>,
) -> Result<Vec<Person>, ServerFnError> {
    auth::logged_in().await?;

    //DB connection
    use crate::app::ssr::*;
    let pool = pool()?;

    println!("{:?},{:?},{:?}", users_change, users_delete, users_add);

    //Update UserFile
    //delete
    let mut array = "?,".repeat(users_delete.len());
    array.pop();
    let query = format!(
        "DELETE FROM userFile WHERE fileID=? and userID IN ({});",
        array
    );
    let mut q = sqlx::query(query.as_str()).bind(&image_id);
    for id in users_delete {
        q = q.bind(id);
    }
    q.execute(&pool).await?;
    //change
    for (old_id, person) in users_change {
        if person.name == "".to_string() {
            continue; // Skip this person.
        }

        let user = match auth::ssr::SqlUser::get_from_username(person.name.clone(), &pool).await {
            Some(u) => u.id,
            None => {
                let res = sqlx::query("INSERT OR IGNORE INTO users (username) VALUES (?)")
                    .bind(person.name)
                    .execute(&pool)
                    .await?;

                res.last_insert_rowid()
            }
        };
        let binds: Vec<Option<u32>> = match person.bounds {
            Some(b) => vec![b.x, b.y, b.w, b.h]
                .into_iter()
                .map(|v| Some(v))
                .collect(),
            None => vec![None::<u32>; 4],
        };
        let mut q = sqlx::query(
            "UPDATE userFile SET userID = ?, x=?, y=?, width=?, height=? WHERE userID = ? and fileID = ?",
        )
        .bind(user);
        for b in binds {
            q = q.bind(b);
        }
        q.bind(old_id).bind(&image_id).execute(&pool).await?;
    }
    //add
    for person in users_add {
        if person.name == "".to_string() {
            continue; // Skip this person.
        }

        let user = match auth::ssr::SqlUser::get_from_username(person.name.clone(), &pool).await {
            Some(u) => u.id,
            None => {
                let res = sqlx::query("INSERT OR IGNORE INTO users (username) VALUES (?)")
                    .bind(person.name)
                    .execute(&pool)
                    .await?;

                res.last_insert_rowid()
            }
        };
        let binds: Vec<Option<u32>> = match person.bounds {
            Some(b) => vec![b.x, b.y, b.w, b.h]
                .into_iter()
                .map(|v| Some(v))
                .collect(),
            None => vec![None::<u32>; 4],
        };
        let mut q = sqlx::query(
            "INSERT OR REPLACE INTO userFile (userID, fileID, x, y, width, height) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(user)
        .bind(&image_id);
        for b in binds {
            q = q.bind(b);
        }
        q.execute(&pool).await?;
    }

    get_users_in_image(image_id).await
}

//Display image and it's deatils
#[component]
pub fn image_view<W>(image_id: W, #[prop(into)] push_delete: Callback<()>) -> impl IntoView
where
    W: Fn() -> String + Copy + 'static,
{
    use crate::components::loading::Loading_Triangle;
    let img = create_rw_signal(None::<ImageDb>);
    let _ = create_resource(image_id, move |value| async move {
        img.set(match get_image(value).await {
            Ok(i) => Some(i),
            Err(_) => None::<ImageDb>,
        });
    });

    let image_info = Signal::derive(move || {
        if let Some(img) = img.get() {
            img.into_info()
        } else {
            ImageInfo::default()
        }
    });
    let (empty, _set_empty) = create_signal("   --".to_string());

    let (editing_image_info, set_editing_image_info) = create_signal(false);

    let (delete_prompt, set_delete_prompt) = create_signal(false);

    view! {
        <Suspense fallback = move|| view!{
            <div class="img_alt">
                <Loading_Triangle show=move||{true}/>
            </div>}>
            <div class="imageview">
                {move || match img.get(){
                    Some(image) =>
                        view!{<img src={format!("data:image/jpeg;base64,{}", image.path)} alt="Base64 Image" class="" />}
                        .into_view(),
                    None =>
                        view!{
                            <div class="img_alt">
                                <Loading_Triangle show=move||{true}/>
                            </div>
                        }.into_view(),
                    }}
            </div>
            <div class="image-info">
                <div class="wraper-h">
                    <div class="people">
                        <h3>"In this picture:"</h3>
                        <UsersInPicture
                            image_id = image_id
                            imgb64 = img
                        />
                    </div>
                    <div class="upload-info">
                        <h3>"Image info:"</h3>
                        <span><Icon class="icon" icon=icondata::FaCameraSolid/>
                            {move ||{if let Some(date) = image_info().created_date {date}else{empty()}}}
                        </span>
                        <span><Icon class="icon" icon=icondata::BiMapSolid/>
                            {move ||if let Some(location) = image_info().location {location}else{empty()}}
                        </span>
                        {
                            let disable = move ||{
                                if image_info().id.is_empty(){
                                    return true;
                                }
                                let user = use_context::<User>();
                                if let Some(user) = user{
                                    return user.username != image_info().uploader &&
                                        !user.has("admin");
                                }
                                return true;
                            };
                            view! {
                                <button
                                    on:click=move |_| {set_editing_image_info(true);}
                                    class:hastooltip=disable
                                    disabled=disable>
                                    <span class="tooltiptext">"You can only edit your own images"</span>
                                    <Icon class="icon" icon=icondata::FaPenSolid/>
                                </button>
                            }
                        }
                        {move || if image_info().id.is_empty().not(){
                            view!{
                                <ImageInfoEdit
                                image=image_info()
                                on_close=move||set_editing_image_info(false)
                                open=editing_image_info
                                update_image=move |new_image_info|{
                                    img.update(|mut img|{
                                        if let Some(ref mut img)= &mut img{
                                            img.created_date=new_image_info.created_date.clone();
                                            img.location=new_image_info.location.clone();
                                        };
                                    });
                                }/>
                            }
                        }else{
                            ().into_view()
                        }}
                    </div>
                    <div class="upload-info">
                        <h3>"Uploaded by:"</h3>
                        <span><Icon class="icon" icon=icondata::BiUserCircleSolid/>
                            {move || if !image_info().uploader.is_empty(){image_info().uploader}else{empty()}}
                        </span>
                        <span><Icon class="icon" icon=icondata::FaCalendarDaysSolid/>
                            {move || if !image_info().upload_date.is_empty(){image_info().upload_date}else{empty()}}
                        </span>
                        {
                            let disable = move||{
                                let user = use_context::<User>();
                                if let Some(user) = user{
                                    return user.username != image_info().uploader &&
                                        !user.has("admin");
                                }
                                return true
                            };

                            move || if !delete_prompt.get() {
                                view!{
                                    <div>
                                    <button
                                        class:hastooltip=disable
                                        disabled=disable
                                        on:click=move |_| {set_delete_prompt(true)}>
                                        <span class="tooltiptext">"You can only delete your own images"</span>
                                        "Delete image" </button>
                                    </div>
                                }
                            } else {
                                view!{
                                    <div>
                                    <button
                                        style="background-color: red;"
                                        on:click=move |_| {
                                            set_delete_prompt(false);
                                            push_delete({});
                                        }>"Delete"</button>
                                    <button style="margin-left: 4px; background-color: gray;" on:click=move |_| {set_delete_prompt(false)}>"Cancel"</button>
                                    </div>
                                }
                            }
                        }
                    </div>
                </div>
            </div>
        </Suspense>
    }
}

//Display icons and names of people in the image
#[component]
fn users_in_picture<W>(image_id: W, imgb64: RwSignal<Option<ImageDb>>) -> impl IntoView
where
    W: Fn() -> String + Copy + 'static,
{
    let people = create_resource(image_id, |image_id| async move {
        let people_res = get_users_in_image(image_id).await;
        if let Ok(people_vec) = people_res {
            people_vec
        } else {
            //handle error
            Vec::new()
        }
    });

    let (editing_people, set_editing_people) = create_signal(false);

    view! {
        <div class="faces">{
            view!{
                <Show when=move||(people.get().is_some() && imgb64.get().is_some())
                fallback = move|| view!{<p>"loading"</p>}>
                {
                    let image = decode_image(imgb64.get().unwrap().path.clone());
                    view!{<For
                        each=move || people.get().unwrap().into_iter().enumerate()
                        key=|(ind, _)| ind.clone()
                        children={
                            let i = image.clone();
                            move |(_index,person)| {
                            view! {
                                <div class="face">
                                    <img
                                        src={format!("data:image/webp;base64,{}", img_from_bounds(&i, person.bounds))}
                                        alt="Base64 Image" />
                                    <span>{person.name}</span>
                                </div>
                            }
                        }
                    }
                    />
                    <button class="edit_persons"
                        disabled=move ||{image_id().is_empty()}
                        on:click=move |_| {
                            set_editing_people(true);
                        }><Icon class="icon" icon=icondata::FaPenSolid/>
                    </button>
                    <UserInImageEdit
                    image_id=image_id()
                    img=image.clone()
                    people=people().unwrap()
                    on_close=move||set_editing_people(false)
                    open=editing_people
                    update_users=move |users|{
                        people.set(users);
                    }/>
                }
                }
                </Show>
            }}
        </div>
    }
}

//Display edit people dialog
#[component]
fn user_in_image_edit<F, W, I>(
    image_id: String,
    img: DynamicImage,
    people: Vec<Person>,
    on_close: F,
    open: W,
    update_users: I,
) -> impl IntoView
where
    F: Fn() + 'static + Clone,
    W: Fn() -> bool + 'static,
    I: Fn(Vec<Person>) + 'static + Clone,
{
    let users = create_rw_signal(vec![]);
    spawn_local(async move {
        match get_user_list_sans_admin().await {
            Ok(u) => users.set(
                u.iter()
                    .map(|user_info| user_info.username.clone())
                    .collect(),
            ),
            Err(_) => (),
        };
    });

    let mut next_person_id = -1;
    let orig_people = people.clone();
    let (people, set_people) = create_signal(people);
    let (changed, set_changed) = create_signal(false);
    let (updating_users, set_updating_users) = create_signal(false);
    let (update_error, set_update_error) = create_signal(None::<String>);
    let (delete_people, set_delete_people) = create_signal(Vec::new());

    let on_close_clone = on_close.clone();
    let on_close_click = move |_| on_close_clone();

    let on_close_clone = on_close.clone();
    let on_edit_save = move |_| {
        //check for changes
        if !changed.get_untracked() {
            on_close_clone();
            return;
        }
        //check for duplicate names
        let mut names = HashSet::new();
        set_update_error(None);
        for person in people.get_untracked().into_iter() {
            if person.name.len() > 0 {
                if names.contains(&person.name) {
                    set_update_error(Some(format!(
                        "Names must be unique, but {} appears at least twice",
                        person.name
                    )));
                    return;
                }
                names.insert(person.name);
            } else {
                set_update_error(Some(format!(
                    "All names must be set. Remove unneeded fields."
                )));
                return;
            }
        }
        //save changes
        set_updating_users(true);
        let update_users = update_users.clone();
        let image_id = image_id.clone();
        let on_close = on_close_clone.clone();
        let mut add_people = Vec::new();
        let mut change_people: Vec<(i64, Person)> = Vec::new();
        for person in people.get_untracked().iter() {
            if person.id < 0 {
                add_people.push(person.clone());
            } else {
                let orig = orig_people.iter().find(|p| p.id == person.id).unwrap();
                if person.name != orig.name {
                    change_people.push((person.id.clone(), person.clone()));
                }
            }
        }
        spawn_local(async move {
            logging::log!(
                "{:?},{:?},{:?}",
                change_people,
                delete_people.get_untracked(),
                add_people
            );
            match update_users_in_image(
                image_id,
                change_people,
                delete_people.get_untracked(),
                add_people,
            )
            .await
            {
                Ok(new_people) => {
                    set_updating_users(false);
                    on_close();
                    update_users(new_people);
                }
                Err(e) => {
                    set_update_error(Some(format!("An Error occured{}", e)));
                    set_updating_users(false);
                }
            }
        });
    };

    view! {
        <Dialog
            on_close=on_close
            open=open
            close_on_outside=false
            close_button=false
            small=true>
            {
                view!{
                    <div>
                        <h3> Edit who is visible in the image: </h3>
                        <br/>
                        <div class="faces">
                            <For
                                each=people
                                key=|person| person.id.clone()
                                children={
                                    let i = img.clone();
                                    move |person| {
                                    let (name, set_name) = create_signal(
                                        if person.name.len() == 0 {
                                            None
                                        }else{
                                            Some(person.name.clone())
                                        });
                                    let _ = create_resource(
                                        name,
                                        // every time `get_name` changes, this will run
                                        move |value| async move {
                                            let v = match value {
                                                Some(v) => v,
                                                None => return,
                                            };
                                            set_people.update(|p| {
                                                p.iter_mut().find(|p|{
                                                    p.id == person.id
                                                }).unwrap().name = v;
                                            });
                                        },
                                    );
                                    view! {
                                        <div class="face">
                                        <img
                                            src={format!("data:image/webp;base64,{}", img_from_bounds(&i, person.bounds))}
                                            alt="Base64 Image" />
                                            <OptionalSelect class="person"
                                                options=users
                                                search_text_provider=move |o: String| o
                                                render_option=move |o: String| o
                                                selected = name
                                                add=move |v: String| users.update(|users| {
                                                    set_changed(true);
                                                    users.push(v.clone());
                                                    set_name(Some(v));
                                                })
                                                set_selected=move|name|{
                                                    set_changed(true);
                                                    set_name(name);
                                                }
                                                allow_deselect=false
                                            />
                                            <button on:click=move |_| {
                                                set_changed(true);
                                                set_people.update(|people| {
                                                    people.retain(|p| {
                                                        if p.id == person.id {
                                                            if p.id > 0{
                                                                set_delete_people.update(|delete|{
                                                                    delete.push(p.id);
                                                                });
                                                            }
                                                        }
                                                        p.id != person.id
                                                    })
                                                });
                                                name.dispose();
                                            }>{"Remove"}</button>
                                        </div>
                                    }
                                }
                                }
                                />
                                <button class="edit_persons"
                                    on:click=move |_| {
                                        set_changed(true);
                                        set_people.update(move |people|{
                                            people.push(Person{
                                                id: next_person_id.clone(),
                                                bounds: None,
                                                name: "".to_string(),
                                        });
                                        });
                                        next_person_id-=1;
                                    }><Icon class="icon" icon=icondata::FaPlusSolid/>
                                </button>
                            </div>
                        <Show when=move||{update_error().is_some()}>
                            <span>{update_error().unwrap()}</span>
                        </Show>
                        <br/>
                        <div class="bottom-buttons">
                            <button type="button" on:click=on_close_click.clone()>
                                "Cancel"
                            </button>
                            <button type="button"
                            on:click=on_edit_save.clone()>
                                {if updating_users.get() {"Loading..."} else {"Save"}}
                            </button>
                        </div>
                    </div>
                }
            }
        </Dialog>
    }
}

//Display edit image info dialog
#[component]
fn image_info_edit<F, W, I>(
    image: ImageInfo,
    on_close: F,
    open: W,
    update_image: I,
) -> impl IntoView
where
    F: Fn() + 'static + Clone,
    W: Fn() -> bool + 'static,
    I: Fn(ImageInfo) + 'static + Clone,
{
    let (updating_image_info, set_updating_image_info) = create_signal(false);
    let (update_error, set_update_error) = create_signal(None);
    let input_location = create_node_ref::<Input>();
    let input_created_date = create_node_ref::<Input>();

    let on_close_clone = on_close.clone();
    let on_close_click = move |_| on_close_clone();

    let image_clone = image.clone();
    let on_close_clone = on_close.clone();
    let on_edit_save = move |_| {
        let node_created_date = input_created_date
            .get()
            .expect("ref should be loaded by now");
        let node_loaction = input_location.get().expect("ref should be loaded by now");
        let location = if node_loaction.value().is_empty() {
            None
        } else {
            Some(node_loaction.value())
        };
        let created_date = if node_created_date.value().is_empty() {
            None
        } else {
            Some(node_created_date.value())
        };
        //check for changes
        if image_clone.created_date == created_date && image_clone.location == location {
            on_close_clone();
            return;
        }
        //save changes
        set_updating_image_info(true);
        let mut new_img = image_clone.clone();
        new_img.created_date = created_date.clone();
        new_img.location = location.clone();
        let image_id = image_clone.id.clone();
        let update_image = update_image.clone();
        let on_close = on_close_clone.clone();
        spawn_local(async move {
            match update_image_info(image_id.clone(), created_date, location).await {
                Ok(_) => {
                    set_updating_image_info(false);
                    update_image(new_img);
                    on_close();
                }
                Err(e) => {
                    set_update_error(Some(e));
                    set_updating_image_info(false);
                }
            }
        });
    };

    view! {
        <Dialog
            on_close=on_close
            open=open
            close_on_outside=false
            close_button=false
            small=true>
            <form>
                <h3> Edit the image information: </h3>
                <br/>
                <label for="created_date"><Icon class="icon" icon=icondata::FaCameraSolid/>Taken on</label>
                <input
                    _ref=input_created_date
                    type="date"
                    value={if let Some(date) = image.created_date.clone() {date}else{"".to_string()}}
                    name="created_date"
                />
                <br/>
                <label for="created_date"><Icon class="icon" icon=icondata::BiMapSolid/>Location</label>
                <input
                    _ref=input_location
                    type="text"
                    value={if let Some(location) = image.location.clone() {location}else{"".to_string()}}
                    name="loaction"
                />
                <br/>
                <Show when=move||{update_error().is_some()}>
                    <span>{format!("An Error occured{}", update_error().unwrap())}</span>
                </Show>
                <div class="bottom-buttons">
                    <button type="button" on:click=on_close_click.clone()>
                        "Cancel"
                    </button>
                    <button type="button"
                    on:click=on_edit_save.clone()>
                        {if updating_image_info.get() {"Loading..."} else {"Save"}}
                    </button>
                </div>
            </form>
        </Dialog>
    }
}
