use leptos::*;

#[component]
pub fn Dialog<F>(
    /// `children` takes the `Children` type
    children: ChildrenFn,
    open: ReadSignal<bool>,
    on_close: F) -> impl IntoView
where
    F: Fn() + 'static+ Clone
{
    let close = move |_|{on_close()};
    
    view! {
        <Show when=open>
            <div class="modal">
                <div class="modal-content">
                <div class="close" 
                on:click = close.clone()>
                    <i class="fas fa-times-circle"></i>
                </div>
                {children().into_view()}
                </div>
            </div>
        </Show>
    }
}