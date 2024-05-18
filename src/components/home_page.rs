use leptos::html::Input;
use leptos::html::Select;
use leptos::*;
use crate::components::feed::InfiniteFeed;
use crate::components::upload::UploadMedia;
use crate::components::dialog::Dialog;

#[derive(Clone, Debug, PartialEq)]
pub struct Filters {
    pub tags: Option<(String, Vec<String>)>,
    pub people: Option<(String, Vec<i64>)>,
}

#[component]
pub fn HomePage() -> impl IntoView
{
    let (showing_upload, set_showing_upload) = create_signal(false);

    let filter_options = vec![
        "HAS".to_string(),
        "ONLY".to_string(),
        "NOT".to_string(),
    ];

    let (filter, set_filter) = create_signal(Filters {
        tags: None,
        people: None,
    });



    let select_ref = create_node_ref::<Select>();
    let select_ref_2 = create_node_ref::<Select>();
    let input_ref = create_node_ref::<Input>();
    let input_ref_2 = create_node_ref::<Input>();

    view! {
        <button
            class = "floating displayFeed"
            on:click=move |_| {
                logging::log!("Open upload dialog"); 
                set_showing_upload(true);
            }><i class="fas fa-plus"></i>
        </button>
        <Dialog 
            on_close=move || set_showing_upload(false) 
            open=showing_upload>
            <h1>"Upload"</h1>
            <UploadMedia/>
        </Dialog>

        <div class="filtering">
        <select
            _ref=select_ref
            >
            {filter_options.iter().map(|option| {
                view! {
                    <option value={option.clone()}>{option}</option>
                }
            }).collect::<Vec<_>>()}
        </select>
        <input _ref=input_ref type="text" placeholder="Filter by tags (comma separated)"/>
        <select
        _ref=select_ref_2
        >
        {filter_options.iter().map(|option| {
            view! {
                <option value={option.clone()}>{option}</option>
            }
        }).collect::<Vec<_>>()}
        </select>
        <input _ref=input_ref_2 type="text" placeholder="Filter by people (comma separated)"/>
        <button
            on:click=move |_| {
                let filter = select_ref.get().unwrap().value();
                let filter_2 = select_ref_2.get().unwrap().value();
                let i_tags = input_ref.get().unwrap().value();
                let i_people = input_ref_2.get().unwrap().value();

                let valid_tags = i_tags.split(",").map(|tag| tag.trim().to_string()).collect::<Vec<String>>();      
                let valid_people = i_people.split(",").map(|person| person.trim().parse::<i64>().unwrap_or(0)).collect::<Vec<i64>>();

                let mut valid_tag_filter: Option<(String, Vec<String>)> = Some((filter, valid_tags.clone()));
                let mut valid_people_filter: Option<(String, Vec<i64>)> = Some((filter_2, valid_people.clone()));

                
                if valid_tags.is_empty() || valid_tags.iter().all(|tag| tag.is_empty()){
                    valid_tag_filter = None;
                }
                
                if valid_people.is_empty() || valid_people.iter().all(|person| *person == 0){
                    valid_people_filter = None;
                }

                set_filter(Filters {
                    tags: valid_tag_filter,
                    people: valid_people_filter,
                });
        
            }
            >"Filter"
        </button>
        </div>
        <InfiniteFeed filter=filter/>
        }
}