#[cfg(feature = "ssr")]
use crate::auth;
use leptos::*;
use serde::Deserialize;
use serde::Serialize;
#[cfg(feature = "ssr")]
use std::fs::File;
#[cfg(feature = "ssr")]
use std::io::Read;
use crate::components::dialog::Dialog;
use leptos::html::Input;
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
    pub fn into_info(
        self,
    ) -> ImageInfo {
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
pub async fn update_image_info(image_id: String, created_date: Option<String>, location: Option<String>) -> Result<(), ServerFnError> {
    let user = auth::logged_in().await?;

    //DB connection
    use crate::app::ssr::*;
    let pool = pool()?;

    //only uploader
    let uploader:bool = sqlx::query_scalar("SELECT uploadedBy=? FROM files WHERE id = ?")
        .bind(user.id)
        .bind(image_id.clone())
        .fetch_one(&pool)
        .await?;

    if !uploader {
        return Err(ServerFnError::ServerError(
            "You are not authorized, only the uploader can change an image".to_string(),
        ))
    }

    //Check if created_date is a valid date
    use regex::Regex;
    if let Some(date) = created_date.clone() {
        let valid_date = Regex::new(r"^\d{4}-(0[1-9]|1[012])-(0[1-9]|[12][0-9]|3[01])$").unwrap().is_match(&date);
        if !valid_date{
            return Err(ServerFnError::ServerError(
                "The date is corrupted. The date must have the format yyyy-mm-dd".to_string(),
            ))
        }
    }

    //Update image
    sqlx::query(
        "UPDATE files SET createdDate=?,location=? WHERE id = ?;",
    )
    .bind(created_date)
    .bind(location)
    .bind(image_id)
    .execute(&pool)
    .await?;

    Ok(())
}

//Display image and it's deatils
#[component]
pub fn image_view<W>(image_id: W) -> impl IntoView
where
    W: Fn() -> String + 'static,
{
    use crate::components::loading::Loading_Triangle;
    let image = create_resource(image_id, get_image);
    let image_info = Signal::derive(move||
        if let Some(Ok(img))=image.get(){
            img.into_info()
        }else{
            ImageInfo::default()
        });
    let people = create_resource(move || (), |_| async { vec![1, 2, 3, 4] });
    let (empty, _set_empty) = create_signal("   --".to_string());

    let (editing_image_info, set_editing_image_info) = create_signal(false);

    view! {
        <Suspense fallback = move|| view!{
            <div class="img_alt">
                <Loading_Triangle show=move||{true}/>
            </div>}>
            <div class="imageview">
                {move || match image.get(){
                    Some(Ok(image)) =>
                        view!{<img src={format!("data:image/jpeg;base64,{}", image.path)} alt="Base64 Image" class="" />}
                        .into_view(),
                    None =>
                        view!{
                            <div class="img_alt">
                                <Loading_Triangle show=move||{true}/>
                            </div>
                        }.into_view(),
                    Some(Err(e)) =>
                        view!{
                            <h1>"An Error occured"</h1>
                            <span>{format!("An Error occured{}", e)}</span>
                        }.into_view(),
                }}
            </div>
            <div class="image-info">
                <div class="wraper-h">
                    <div class="people">
                        <h3>"In this picture:"</h3>
                        <div class="faces">
                            <Suspense fallback = move|| view!{<p>"loading"</p>}>
                                <Show when=move||people.get().is_some()>
                                    <For
                                        each=move || people.get().unwrap()
                                        key=|person| person.clone()
                                        children=move |person: i32| {
                                            view! {
                                                <div class="face">
                                                    <img src={format!(" data:image/webp;base64,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")} alt="Base64 Image" />
                                                    <span>{format!("Name {}", person)}</span>
                                                </div>
                                            }
                                        }
                                    />
                                    <button class="edit_persons"
                                        on:click=move |_| {
                                            logging::log!("Edit");
                                        }><i class="fas fa-pen"></i>
                                    </button>
                                </Show>
                            </Suspense>
                        </div>
                    </div>
                    <div class="upload-info">
                        <h3>"Image info:"</h3>
                        <span><i class="fas fa-camera"></i>
                            {move ||{if let Some(date) = image_info().created_date {date}else{empty()}}}
                        </span>
                        <span><i class="fas fa-map-marker-alt"></i>
                            {move ||if let Some(location) = image_info().location {location}else{empty()}}
                        </span>
                        <button on:click=move |_| {set_editing_image_info(true);}><i class="fas fa-pen"></i>"Edit"</button>
                        {move || if image_info().id.is_empty().not(){
                            view!{
                                <ImageInfoEdit
                                image=image_info()
                                on_close=move||set_editing_image_info(false)
                                open=editing_image_info
                                update_image=move |new_image_info|{
                                    image.update(|mut image|{
                                        if let Some(Ok(ref mut img))= &mut image{
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
                        <span><i class="fas fa-user-circle"></i>
                            {move || if !image_info().uploader.is_empty(){image_info().uploader}else{empty()}}    
                        </span>
                        <span><i class="fas fa-calendar-day"></i>
                            {move || if !image_info().upload_date.is_empty(){image_info().upload_date}else{empty()}}
                        </span>
                        <button>"Delete image"</button>
                    </div>
                </div>
            </div>
        </Suspense>
    }
}


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
    let on_close_click = move |_|{on_close_clone()};

    let image_clone = image.clone();
    let on_close_clone = on_close.clone();
    let on_edit_save = move |_| {
        let node_created_date = input_created_date.get().expect("ref should be loaded by now");
        let node_loaction = input_location.get().expect("ref should be loaded by now");
        let location = if node_loaction.value().is_empty() {None} else {Some(node_loaction.value())};
        let created_date = if node_created_date.value().is_empty() {None} else {Some(node_created_date.value())};
        //check for changes
        if image_clone.created_date==created_date &&
        image_clone.location==location {
            on_close_clone();
            return
        }
        //save changes
        set_updating_image_info(true);
        let mut new_img = image_clone.clone();
        new_img.created_date=created_date.clone();
        new_img.location=location.clone();
        let image_id = image_clone.id.clone();
        let update_image = update_image.clone();
        let on_close = on_close_clone.clone();
        spawn_local(async move{
            match update_image_info(image_id.clone(), created_date, location).await{
                Ok(_)=> {
                    set_updating_image_info(false);
                    update_image(new_img);
                    on_close();},
                Err(e) => {
                    set_update_error(Some(e));
                    set_updating_image_info(false);},
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
                <label for="created_date"><i class="fas fa-camera"></i>Taken on</label>
                <input
                    _ref=input_created_date
                    type="date"
                    value={if let Some(date) = image.created_date.clone() {date}else{"".to_string()}}
                    name="created_date"
                />
                <br/>
                <label for="created_date"><i class="fas fa-map-marker-alt"></i>Location</label>
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
