use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
extern crate rand; 
use rand::Rng;
use leptos::html::Div;
use leptos_use::{UseInfiniteScrollOptions, use_infinite_scroll_with_options};



#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/photo-album.css"/>

        // sets the document title
        <Title text="photo-album"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <TopBar/>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="test" view=TestPage/>
                    <Route path="hello" view=HelloPage/>
                </Routes>
            </main>
        </Router>
    }
}

// div picture component
#[component]
fn PictureDiv() -> impl IntoView {
    view!{
        <div></div>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {

    view! {
        <h1>"Home"</h1>
        // <DynamicList initial_length=5 initial_period=1/>
        <InfiniteFeed/>
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Image {
    src: String,
    date: String,
}

fn fetch_image(num: usize) -> Option<Image> {
// fn fetch_image() -> Vec<(i32,Vec<(i32,Vec<(i32,Vec<Image>)>)>)> {

    //TEST DATA
    //======================================================
    let mut images = vec![
        Image {
            src: "pic1.jpg".to_string(),
            date: "2024-01-15".to_string(),
        },
        Image {
            src: "pic1.jpg".to_string(),
            date: "2024-01-15".to_string(),
        },
        Image {
            src: "pic11.jpg".to_string(),
            date: "2024-01-15".to_string(),
        },
        Image {
            src: "pic111.jpg".to_string(),
            date: "2024-01-15".to_string(),
        },
        Image {
            src: "pic1111.jpg".to_string(),
            date: "2024-01-15".to_string(),
        },
        Image {
            src: "pic11111.jpg".to_string(),
            date: "2024-01-15".to_string(),
        },
        Image {
            src: "pic2.jpg".to_string(),
            date: "2024-02-20".to_string(),
        },
        Image {
            src: "pic3.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic33.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic333.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic3333.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic33333.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic31.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic313.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic31313.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic32.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic323.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic3232.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic322223.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic321233.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic333133333.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic30.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic39.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic388.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic354.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic4.jpg".to_string(),
            date: "2024-03-05".to_string(),
        },
        Image {
            src: "pic5.jpg".to_string(),
            date: "2023-02-08".to_string(),
        },
        Image {
            src: "pic6.jpg".to_string(),
            date: "2023-01-21".to_string(),
        },
        Image {
            src: "pic7.jpg".to_string(),
            date: "2024-03-10".to_string(),
        },
        Image {
            src: "pic8.jpg".to_string(),
            date: "2024-02-15".to_string(),
        },
        Image {
            src: "pic9.jpg".to_string(),
            date: "2024-01-10".to_string(),
        },
        Image {
            src: "pic10.jpg".to_string(),
            date: "2024-03-01".to_string(),
        },
    ];
    //=====================================================


    //Sort images by date (yyyy-mm-dd)
    images.sort_by(|a, b| a.date.cmp(&b.date));

    //Check if valid parameter
    if num < images.len() {
        Some(images[num].clone())
    } else {
        //If out of bounds
        None
    }


    // //Vec<(Year, Vec<(Month, Vec<Day, Vec<Image>)>)>)>
    // let mut sorted_images: Vec<(i32, Vec<(i32, Vec<(i32, Vec<Image>)>)>)> = Vec::new();

    // //Iterate the list and sort by year, then month, then day
    // for image in &images {
    //     //Parse dates to i32
    //     let year = image.date[..4].parse::<i32>().unwrap();
    //     let month = image.date[5..7].parse::<i32>().unwrap();
    //     let day = image.date[8..10].parse::<i32>().unwrap();

    //     //Check if the year, month or day exists, to ensure there is no repetition
    //     if let Some((_, months)) = sorted_images.iter_mut().find(|(y, _)| *y == year) {
    //         if let Some((_, days)) = months.iter_mut().find(|(m, _)| *m == month) {
    //             if let Some((_, images_for_day)) = days.iter_mut().find(|(d, _)| *d == day) {
    //                 images_for_day.push(image.clone());
    //             } else {
    //                 days.push((day, vec![image.clone()]));
    //                 days.sort_by_key(|(d, _)| *d);
    //             }
    //         } else {
    //             months.push((month, vec![(day, vec![image.clone()])]));
    //             months.sort_by_key(|(m, _)| *m);
    //         }
    //     } else {
    //         sorted_images.push((year, vec![(month, vec![(day, vec![image.clone()])])]));
    //         sorted_images.sort_by_key(|(y, _)| *y);
    //     }
    // }

    //Not needed, but keeping it beacuse Im scared to delete it :(
    // sorted_images.sort_by_key(|(y, _)| *y);
        
    // sorted_images
}


#[component]
fn infinite_feed() -> impl IntoView {
    let el = create_node_ref::<Div>();

    let mut next_image_number = 0;
    let mut image_vector = Vec::new();  
    image_vector.push(fetch_image(next_image_number));
    let (feed, set_feed) = create_signal(image_vector);
    
    let _ = use_infinite_scroll_with_options(
        el,
        move |_| async move {
            //Get len of vec
            let len = feed.with_untracked(|d| d.len());
            //Check for next image and push to feed
            if let Some(next_image) = fetch_image(len+1) {
                set_feed.update(|feed| feed.push(Some(next_image)));
            }
        },
        UseInfiniteScrollOptions::default().distance(10.0),
    );

    view!{
        //Year
        <div class="flowdiv" node_ref=el>
        <For 
          each=feed
          key=|photokey| photokey.clone() 
          let:block>
          {block.unwrap().src}
          <img src="sss" 
                style:height=rand::thread_rng().gen_range(250..350).to_string()+"px"
                style:width=rand::thread_rng().gen_range(250..350).to_string()+"px"
                />
        </For>
        </div>
        // <For 
        //   each=photos 
        //   key=|yearkey| yearkey.clone() 
        //   let:year>
        //     <br/>
        //     <br/>
        //     {year.0}
        //     <br/>
        //     //Month
        //     <For 
        //       each=move||{year.1.clone()}
        //       key=|monthkey| monthkey.clone()
        //       let:month>
        //         <br/>
        //         {match month.0 {
        //             1 => "January",
        //             2 => "February",
        //             3 => "March",
        //             4 => "April",
        //             5 => "May",
        //             6 => "June",
        //             7 => "July",
        //             8 => "August",
        //             9 => "September",
        //             10 => "October",
        //             11 => "November",
        //             12 => "December",
        //             _ => ""
        //         }}
        //         <br/>
        //         //Day
        //         <For 
        //           each=move||{month.1.clone()}
        //           key=|daykey| daykey.clone()
        //           let:day>
        //             // --{day.0} 
        //             //Vec of images in one day
        //             <For 
        //               each=move||{day.1.clone()}
        //               key=|imgkey| imgkey.clone()
        //               let:img>
        //                 <img src={img.src} 
        //                 style:width=rand::thread_rng().gen_range(150..350).to_string()+"px"
        //                 style:height=rand::thread_rng().gen_range(150..350).to_string()+"px"
        //                 />
        //             </For>
        //         </For>
        //     </For>
        // </For>


        
    }
}


#[component]
fn TestPage() -> impl IntoView {
    let el = create_node_ref::<Div>();

    let (data, set_data) = create_signal(vec![1, 2, 3, 4, 5, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7]);

    let _ = use_infinite_scroll_with_options(
        el,
        move |_| async move {
            let len = data.with_untracked(|d| d.len());
            set_data.update(|data| *data = (1..len + 6).collect());
        },
        UseInfiniteScrollOptions::default().distance(10.0),
    );

    view! {
        <div
        class="flowdiv"
            node_ref=el
        >
            <ul>
            <For each=move || data.get() key=|i| *i let:item>
                <li>{item}</li>
            </For>
            </ul>
        </div>
    }
}

#[component]
fn HelloPage() -> impl IntoView {
    
    let (name, set_name) = create_signal("Potato".to_string());
    
    view! {
        <h1>"Hello"</h1>
        <input type="text"
        on:input=move |ev| {
            set_name(event_target_value(&ev));
        }
        prop:value=name
        />
        <p>"Name is: " {name}</p>
    }
}

#[component]
fn TopBar() -> impl IntoView {

    // All routes accessible from navigation bar
    view! {
        <nav>
            <a href="/">"Family Album"</a> // TODO Set to Admin defined name
            <a href="test">"Test"</a>
            <a href="hello">"Hello"</a>
        </nav>
    }
}