use leptos::*;

#[component]
pub fn Dialog<F, W>(
    /// `children` takes the `Children` type
    children: ChildrenFn,
    open: W,
    on_close: F,
) -> impl IntoView
where
    F: Fn() + 'static + Clone,
    W: Fn() -> bool + 'static,
{
    let close = move |_| on_close();

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
