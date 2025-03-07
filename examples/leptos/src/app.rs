use futures::stream::StreamExt;
use leptos::{ev::MouseEvent, *};
use std::rc::Rc;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="container">
            <div>
                <h2>"core"</h2>
                <Core/>
            </div>

            <div>
                <h2>"events"</h2>
                <Events/>
            </div>

            <div>
                <h2>"window"</h2>
                <Window/>
            </div>

            <div>
                <h2>"menu"</h2>
                <Menu/>
            </div>
        </main>
    }
}

#[component]
fn Core() -> impl IntoView {
    let (convert_path, set_convert_path) = create_signal("".to_string());
    let (converted_path, set_converted_path) = create_signal("".to_string());

    let do_convert_path = move |_| {
        let converted = tauri_api::core::convert_file_src(convert_path());
        set_converted_path(converted);
    };

    view! {
        <div>
            <div>
                <label>
                    "Convert path"
                    <input
                        prop:value=convert_path
                        on:input=move |e| set_convert_path(event_target_value(&e))
                    />
                </label>
                <button on:click=do_convert_path>"Convert"</button>
            </div>
            <div>{converted_path}</div>
        </div>
    }
}

#[component]
fn Events() -> impl IntoView {
    let (listen_event, set_listen_event) = create_signal(None);
    let (emit_count, set_emit_count) = create_signal(0);

    spawn_local(async move {
        let mut listener = tauri_api::event::listen::<i32>("event::listen")
            .await
            .unwrap();

        while let Some(event) = listener.next().await {
            tracing::debug!(?event);
            let tauri_api::event::Event {
                event: _,
                id: _,
                payload,
            } = event;
            set_listen_event.set(Some(payload));
        }
    });

    spawn_local(async move {
        let mut listener = tauri_api::event::listen::<i32>("event::emit")
            .await
            .unwrap();

        while let Some(event) = listener.next().await {
            tracing::debug!(?event);
            let tauri_api::event::Event {
                event: _,
                id: _,
                payload,
            } = event;
            set_emit_count.set(payload);
        }
    });

    let trigger_listen_events = move |_| {
        spawn_local(async move {
            tauri_api::core::invoke::<()>("trigger_listen_events", &()).await;
        });
    };

    let trigger_emit_event = move |_| {
        spawn_local(async move {
            tauri_api::event::emit("event::emit", &emit_count.with_untracked(|n| n + 1))
                .await
                .unwrap();
        });
    };

    view! {
        <div>
            <div>
                <button on:click=trigger_listen_events>"Trigger listen events"</button>
                <div>
                    <strong>"Last listen event: "</strong>
                    {move || listen_event()}
                </div>
            </div>

            <div>
                <button on:click=trigger_emit_event>"Trigger emit event"</button>
                <div>
                    <strong>"Events emitted: "</strong>
                    {move || emit_count()}
                </div>
            </div>
        </div>
    }
}

#[component]
fn Window() -> impl IntoView {
    view! {
        <div>
            <div>
                <h3>"Windows"</h3>
                <WindowWindows/>
            </div>

            <div>
                <h3>"Monitors"</h3>
                <WindowMonitors/>
            </div>

            <div>
                <h3>"Events"</h3>
                <WindowEvents/>
            </div>
        </div>
    }
}

#[component]
fn WindowWindows() -> impl IntoView {
    let current_window = create_action(|_| async move { tauri_api::window::get_current() });
    let all_windows = create_action(|_| async move { tauri_api::window::get_all() });

    let refresh = move |_| {
        current_window.dispatch(());
        all_windows.dispatch(());
    };

    current_window.dispatch(());
    all_windows.dispatch(());

    view! {
        <div>
            <div style="display: flex; justify-content: center; gap: 10px;">
                <div>"Current window:"</div>
                {move || {
                    current_window
                        .value()
                        .with(|window| match window {
                            None => "Loading".to_string(),
                            Some(window) => window.label().clone(),
                        })
                }}

            </div>
            <div style="display: flex; justify-content: center; gap: 10px;">
                <div>"All windows:"</div>
                {move || {
                    all_windows
                        .value()
                        .with(|windows| match windows {
                            None => "Loading".to_string(),
                            Some(windows) => {
                                let out = windows
                                    .iter()
                                    .map(|window| { window.label().clone() })
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                format!("[{out}]")
                            }
                        })
                }}

            </div>
            <button on:click=refresh>"Refresh"</button>
        </div>
    }
}

#[component]
fn WindowMonitors() -> impl IntoView {
    let current_monitor =
        create_action(|_| async move { tauri_api::window::current_monitor().await });

    let primary_monitor =
        create_action(|_| async move { tauri_api::window::primary_monitor().await });

    let available_monitors =
        create_action(|_| async move { tauri_api::window::available_monitors().await });

    let monitor_from_point = create_action(|(x, y): &(isize, isize)| {
        let x = x.clone();
        let y = y.clone();
        async move { tauri_api::window::monitor_from_point(x, y).await }
    });

    // let cursor_position =
    //     create_action(|_| async move { tauri_api::window::cursor_position().await });

    let refresh = move |_| {
        current_monitor.dispatch(());
        primary_monitor.dispatch(());
        available_monitors.dispatch(());
    };

    let oninput_monitor_from_point = move |e| {
        let value = event_target_value(&e);
        let Some((x, y)) = value.split_once(',') else {
            return;
        };

        let Ok(x) = x.parse::<isize>() else {
            return;
        };

        let Ok(y) = y.parse::<isize>() else {
            return;
        };

        monitor_from_point.dispatch((x, y));
    };

    current_monitor.dispatch(());
    primary_monitor.dispatch(());
    available_monitors.dispatch(());

    view! {
        <div>
            <div>
                <div style="display: flex; justify-content: center; gap: 10px;">
                    <div>"Current monitor:"</div>
                    {move || {
                        current_monitor
                            .value()
                            .with(|monitor| match monitor {
                                None => "Loading".into_view(),
                                Some(Some(monitor)) => view! { <Monitor monitor/> }.into_view(),
                                Some(None) => "Could not detect monitor.".into_view(),
                            })
                    }}

                </div>
                <div style="display: flex; justify-content: center; gap: 10px;">
                    <div>"Primary monitor:"</div>
                    {move || {
                        primary_monitor
                            .value()
                            .with(|monitor| match monitor {
                                None => "Loading".into_view(),
                                Some(Some(monitor)) => view! { <Monitor monitor/> }.into_view(),
                                Some(None) => "Could not detect monitor.".into_view(),
                            })
                    }}

                </div>
                <div style="display: flex; justify-content: center; gap: 10px;">
                    <div>"Available monitors:"</div>
                    {move || {
                        available_monitors
                            .value()
                            .with(|monitors| match monitors {
                                None => "Loading".into_view(),
                                Some(monitors) => {
                                    view! {
                                        {monitors
                                            .iter()
                                            .map(|monitor| view! { <Monitor monitor/> })
                                            .collect::<Vec<_>>()}
                                    }
                                        .into_view()
                                }
                            })
                    }}

                </div>
                <button on:click=refresh>"Refresh"</button>
            </div>
            <div>
                <label>"Monitor from point" <input on:input=oninput_monitor_from_point/></label>
                <div style="margin: 0 auto;">
                    {move || {
                        monitor_from_point
                            .value()
                            .with(|monitor| match monitor {
                                None => "Enter an `x, y` coordinate.".into_view(),
                                Some(Some(monitor)) => view! { <Monitor monitor/> }.into_view(),
                                Some(None) => "Could not detect monitor.".into_view(),
                            })
                    }}

                </div>
            </div>

            <div>
                // {move || {
                // cursor_position
                // .value()
                // .with(|position| {
                // position
                // .as_ref()
                // .map(|position| {
                // view! {
                // {position.x()}
                // ", "
                // {position.y()}
                // }
                // })
                // })
                // }}
                <div>"Cursor position: "</div>
                <div style="width: 50vw; height: 30vh; margin: 0 auto; border: 2px solid black; border-radius: 5px;">
                    // on:mousemove=move |_| cursor_position.dispatch(())
                    "TODO (See https://github.com/tauri-apps/tauri/issues/10340)"
                </div>
            </div>
        </div>
    }
}

#[component]
fn WindowEvents() -> impl IntoView {
    use tauri_api::window::{DragDropEvent, DragDropPayload, DragOverPayload};

    let (count, set_count) = create_signal(0);
    let increment_count = create_action(|count: &usize| {
        let count = count.clone();
        let window = tauri_api::window::get_current();
        async move {
            web_sys::console::debug_1(&"0".into());
            window.emit("count", count).await.unwrap();
        }
    });

    let (drag_drop, set_drag_drop) = create_signal(().into_view());

    spawn_local(async move {
        let mut window = tauri_api::window::get_current();
        let mut listener = window.listen::<usize>("count").await.unwrap();
        while let Some(event) = listener.next().await {
            set_count(event.payload);
        }
    });

    spawn_local(async move {
        let window = tauri_api::window::get_current();
        let mut listener = window.on_drag_drop_event().await.unwrap();
        while let Some(event) = listener.next().await {
            match event.payload {
                DragDropEvent::Enter(payload) => {
                    let out = view! {
                        <div>
                            <strong>"Enter"</strong>
                            <div>
                                "Paths: ["
                                {payload
                                    .paths()
                                    .iter()
                                    .map(|path| path.to_string_lossy().to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")} "]"
                            </div>
                            <div>
                                "Position: " {payload.position().x()} ", " {payload.position().y()}
                            </div>
                        </div>
                    };

                    set_drag_drop(out.into_view());
                }
                DragDropEvent::Over(payload) => {
                    let out = view! {
                        <div>
                            <strong>"Over"</strong>
                            <div>
                                "Position: " {payload.position().x()} ", " {payload.position().y()}
                            </div>
                        </div>
                    };

                    set_drag_drop(out.into_view());
                }
                DragDropEvent::Drop(payload) => {
                    let out = view! {
                        <div>
                            <strong>"Drop"</strong>
                            <div>
                                "Paths: ["
                                {payload
                                    .paths()
                                    .iter()
                                    .map(|path| path.to_string_lossy().to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")} "]"
                            </div>
                            <div>
                                "Position: " {payload.position().x()} ", " {payload.position().y()}
                            </div>
                        </div>
                    };

                    set_drag_drop(out.into_view());
                }
                DragDropEvent::Leave => {
                    let out = view! { <strong>"Leave"</strong> };
                    set_drag_drop(out.into_view());
                }
            }
        }
    });

    view! {
        <div>
            <div>
                "Count: " {count}
                <button on:click=move |_| increment_count.dispatch(count() + 1)>"+"</button>
            </div>

            <div>
                <h3>"Drag drop event"</h3>
                <div>{drag_drop}</div>
            </div>
        </div>
    }
}

#[component]
fn Monitor<'a>(monitor: &'a tauri_api::window::Monitor) -> impl IntoView {
    view! {
        <div style="display: inline-block; text-align: left;">
            <div>"Name: " {monitor.name().clone()}</div>
            <div>"Size: " {monitor.size().width()} " x " {monitor.size().height()}</div>
            <div>"Position: " {monitor.position().x()} ", " {monitor.position().y()}</div>
            <div>"Scale: " {monitor.scale_factor()}</div>
        </div>
    }
}

#[component]
fn Menu() -> impl IntoView {
    let (event, set_event) = create_signal::<Option<String>>(None);
    let menu = create_local_resource(
        || (),
        move |_| async move {
            let menu = tauri_api::menu::Menu::with_id("tauri-sys-menu").await;
            let mut item_open = tauri_api::menu::item::MenuItem::with_id("Open", "open").await;
            let mut item_close = tauri_api::menu::item::MenuItem::with_id("Close", "close").await;
            menu.append_item(&item_open).await.unwrap();
            menu.append_item(&item_close).await.unwrap();

            spawn_local(async move {
                let mut listener_item_open = item_open.listen().fuse();
                let mut listener_item_close = item_close.listen().fuse();

                loop {
                    futures::select! {
                        event = listener_item_open.next() => match event{
                            None => continue,
                            Some(event) => set_event(Some((*event).clone())),
                        },
                        event = listener_item_close.next() => match event{
                            None => continue,
                            Some(event) => set_event(Some((*event).clone())),
                        },
                    }
                }
            });

            Rc::new(menu)
        },
    );

    let default_menu = move |e: MouseEvent| {
        spawn_local(async move {
            let menu = tauri_api::menu::Menu::default().await;
        });
    };

    let open_menu = move |e: MouseEvent| {
        let menu = menu.get().unwrap();
        spawn_local(async move {
            menu.popup().await.unwrap();
        });
    };

    view! {
        <div
            on:mousedown=open_menu
            style="margin: auto; width: 50vw; height: 10em; border: 1px black solid; border-radius: 5px;"
        >
            {event}
        </div>
    }
}
