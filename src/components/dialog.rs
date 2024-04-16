use leptos::*;

#[component]
pub fn Dialog(
    /// `children` takes the `Children` type
    children: ChildrenFn,
) -> impl IntoView
{
    let (showing, set_showing) = create_signal(true);

    
    view! {
        <Show when=showing>
            <div class="modal">
                <div class="modal-content">
                <div class="close" 
                on:click = move |_|{println!("clicked"); set_showing(false);}>
                    <i class="fas fa-times-circle"></i>
                </div>
                {children().into_view()}
                </div>
            </div>
        </Show>
    }
}