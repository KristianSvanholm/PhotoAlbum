use leptos::*;
use rand::Rng;
use leptos::html::Div;
use leptos_use::{UseInfiniteScrollOptions, use_infinite_scroll_with_options};



#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Image {
    src: String,
    date: String,
}

const FETCH_IMAGE_COUNT: usize = 20;

fn fetch_images(start: usize, count: usize) -> Vec<Image> {
    let mut images = Vec::new();
    for i in start..start + count {
        let image = Image {
            src: format!("https://picsum.photos/200/300?random={}", i),
            date: 
                format!(
                    "{}-{:02}-{:02}",
                    rand::thread_rng().gen_range(2010..2022),
                    rand::thread_rng().gen_range(1..13),
                    rand::thread_rng().gen_range(1..29)
                ),
        };
        images.push(image);
    }
    images
}

#[component]
pub fn infinite_feed() -> impl IntoView {
    let (images, wImages) = create_signal(Vec::new());
    let (start, wStart) = create_signal(0);

    let el = create_node_ref::<Div>();

    let _ = use_infinite_scroll_with_options(
        el,
        move |_| async move {
            let count = FETCH_IMAGE_COUNT; 
            let newStart = start.get_untracked() + count;
            let newImages = fetch_images(newStart, count);
            wImages.update(|images| images.extend(newImages));
            wStart.set(newStart);
        },
        UseInfiniteScrollOptions::default().distance(250.0),
    );

    view! {
        <div
            class="flowdiv"
            node_ref=el
            style="display: flex; flex-wrap: wrap; gap: 10px;"
            >
            <For each=move || images.get() key=|i| i.clone() let:image>
                <div class="image">
                    <img 
                    src=image.src
                    style=format!("height: {}px; width: {}px;", rand::thread_rng().gen_range(250..300), rand::thread_rng().gen_range(250..300))
                    />
                    <p>{image.date}</p>
                </div>
            </For>
        </div>
    }
}


