use leptos::*;
use serde::Serialize;
use serde::Deserialize;
#[cfg(feature = "ssr")]
use std::fs::File;
#[cfg(feature = "ssr")]
use std::io::Read;
#[cfg(feature = "ssr")]
use crate::auth;

//Image struct for images from DB
#[cfg_attr(feature="ssr", derive(sqlx::FromRow))]
#[derive(Clone, Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct ImageDb {
    id: String,
    path: String,
    upload_date: String,
    created_date: String,
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
        "SELECT id, path, uploadDate AS upload_date, createdDate AS created_date FROM files WHERE id = ?;",
    )
    .bind(image_id)
    .fetch_one(&pool)
    .await?;

    // Read the image file
    let mut file = File::open(&img.path).expect("Failed to open image file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read image file");

    // Encode the image buffer as base64
    let base64_image = base64::encode(&buffer);

    // Generate src attribute value with the base64 image
    img.path = base64_image;

    Ok(img)
}

//Creates an infinite feed of images
#[component]
pub fn image_view(
    //image: ReadSignal<ImageDb>
) -> impl IntoView {
    use crate::components::loading::Loading_Triangle;

    let (ready, set_ready) = create_signal(true);
    let (loading, set_loading) = create_signal(true);
    let image = create_resource(|| (), |_| async { get_image("4655b97b-af2d-40a8-8ecb-37857d425c64".to_string()).await });

    view! {
        <div>
            <div class="imageview">
                <Suspense fallback = move|| view!{<h1>"Loading"</h1>}>{
                    move || match image.get(){
                        Some(Ok(image)) => 
                            view!{<img src={format!("data:image/jpeg;base64,{}", image.path)} alt="Base64 Image" class="" />}
                            .into_view(),
                        None => 
                            view!{<h1>"Loading"</h1>}
                            .into_view(),
                        Some(Err(e)) => 
                            view!{
                                <h1>"An Error occured"</h1>
                                <span>{format!("An Error occured{}", e)}</span>
                            }.into_view(),
                    }}
                </Suspense>
            </div>
        </div>
    }
}
