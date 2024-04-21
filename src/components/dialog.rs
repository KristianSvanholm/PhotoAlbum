use leptos::*;
use web_sys::MouseEvent;
use leptos::html::Div;

#[component]
pub fn Dialog<F, W>(
    /// `children` takes the `Children` type
    children: ChildrenFn,
    open: W,
    on_close: F,
    #[prop(default = false)]
    close_on_outside: bool,
    #[prop(default = true)]
    close_button: bool,
) -> impl IntoView
where
    F: Fn() + 'static + Clone,
    W: Fn() -> bool + 'static,
{
    let on_close_clone= on_close.clone();
    let close = move |_|{on_close()};
    let overlay = create_node_ref::<Div>();
    let close_on_outside = move |_|{if close_on_outside{
        //if mouse_event.target().unwrap()==overlay{
            on_close_clone();
        //}
    }};
    
    view! {
        <Show when=open>
            <div class="modal" node_ref=overlay
            on:click = close_on_outside.clone()>
                <div class="modal-content"
                on:click = |mouse_event:MouseEvent|{mouse_event.stop_propagation();}>
                {if close_button {
                    view!{
                        <div class="close" 
                        on:click = close.clone()>
                            <i class="fas fa-times-circle"></i>
                        </div>
                    }.into_view()
                }else{
                    ().into_view()
                }}
                {children().into_view()}
                </div>
            </div>
        </Show>
    }
}