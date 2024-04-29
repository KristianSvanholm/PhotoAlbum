use leptos::*;
use crate::components::feed::Element;

#[component]
pub fn smooth_feed(content: ReadSignal<Vec<Element>>) -> impl IntoView {

    view! {
        <For each=move || content.get() key=|i| i.clone() let:el>
        {
            match el { 
                Element::ImageDb(item) => view!{
                    <img 
                        src={format!{"data:image/jpeg;base64,{}", item.path}} 
                        alt="Base64 Image" 
                        class="image imageSmooth"
                    />
                },
                _ => view!{<img/>} // I have to set an empty img element here. Not ideal but it's
                                   // working. I could od empty div instead, but that breaks css
            }
        }
        </For>
    }
}
