use std::collections::HashMap;
use std::rc::Rc;

use gloo::file::callbacks::FileReader;
use gloo::file::File;
use gloo_net::http::Request;
use js_sys::Array;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::{DragEvent, Event, FileList, HtmlInputElement};
use yew::function_component;
use yew::html::TargetCast;
use yew::use_effect_with_deps;
use yew::use_state;
use yew::Properties;
use yew::virtual_dom::VNode;
use yew::{html, Callback, Component, Context, Html};

const HUNNIT: char = '\u{1F4AF}';
const NOPE: char = '\u{274C}';

struct FileDetails {
    name: String,
    content: Vec<u8>,
    failed: bool,
}

pub enum Msg {
    Loaded(String, Vec<u8>),
    Failed(String),
    Files(Vec<File>),
}

pub struct App {
    pub readers: HashMap<String, FileReader>,
    files: Vec<Rc<FileDetails>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            readers: HashMap::default(),
            files: Vec::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded(file_name, data) => {
                self.files.push(Rc::new(FileDetails {
                    name: file_name.clone(),
                    failed: false,
                    content: data,
                }));
                self.readers.remove(&file_name);
                true
            }
            Msg::Failed(file_name) => {
                self.files.push(Rc::new(FileDetails {
                    name: file_name.clone(),
                    failed: true,
                    content: vec![],
                }));
                self.readers.remove(&file_name);
                true
            }
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let file_name = file.name();
                    let task = {
                        let link = ctx.link().clone();
                        let file_name = file_name.clone();

                        gloo::file::callbacks::read_as_bytes(&file, move |res| match res {
                            Ok(data) => {
                                trace!("File could be read");
                                link.send_message(Msg::Loaded(file_name, data))
                            }
                            Err(e) => {
                                error!("Unable to read file content: {:?}", e);
                                link.send_message(Msg::Failed(file_name))
                            }
                        })
                    };
                    self.readers.insert(file_name, task);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div id="wrapper">
                <p id="title">{ "Upload Your Files" }</p>
                <label for="file-upload">
                    <div
                        id="drop-container"
                        ondrop={ctx.link().callback(|event: DragEvent| {
                            event.prevent_default();
                            let files = event.data_transfer().unwrap().files();
                            Self::upload_files(files)
                        })}
                        ondragover={Callback::from(|event: DragEvent| {
                            event.prevent_default();
                        })}
                        ondragenter={Callback::from(|event: DragEvent| {
                            event.prevent_default();
                        })}
                    >
                        <i class="fa fa-cloud-upload"></i>
                    </div>
                </label>
                <input
                    id="file-upload"
                    type="file"
                    accept=".csv"
                    multiple={true}
                    onchange={ctx.link().callback(move |e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Self::upload_files(input.files())
                    })}
                />
                <div id="preview-area">
                    <DoPut data={self.files.iter().map(Rc::clone).collect::<Vec<Rc<FileDetails>>>()}/>
                </div>
            </div>
        }
    }
}

impl App {
    fn _view_file(file: &FileDetails) -> Html {
        html! {
            <div class="preview-tile">
                <p class="preview-name">{ file.name.clone() }</p>
                <p>{if file.failed {NOPE} else {HUNNIT}}</p>
            </div>
        }
    }

    fn upload_files(files: Option<FileList>) -> Msg {
        let mut result = Vec::new();

        if let Some(files) = files {
            let files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(File::from);
            result.extend(files);
        }
        Msg::Files(result)
    }
}

#[derive(Properties, PartialEq)]
struct DoPutProps {
    data: Vec<Rc<FileDetails>>,
}

impl PartialEq for FileDetails {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.content == other.content && self.failed == other.failed
    }
}

#[function_component(DoPut)]
fn do_put(props: &DoPutProps) -> Html {
    let succcess = use_state(|| true);
    let mut result = Vec::new();
    for iblob in &props.data {
        let blob = iblob.clone();
        let isuccess = succcess.clone();
        use_effect_with_deps(
            move |_| {
                let iisuccess = isuccess.clone();
                spawn_local(async move {
                    let val: Array = blob
                        .content
                        .iter()
                        .map(|u| JsValue::from(*u))
                        .collect::<Array>();
                    let response = Request::put("/api/data").body(val).send().await;
                    match response {
                        Ok(resp) => {
                            if resp.status() != 204u16 {
                                error!("Received unexpected response: {:?}", resp);
                                iisuccess.set(false);
                            }
                        }
                        Err(e) => {
                            error!("Sending to server failed : {:?}", e);
                            iisuccess.set(false);
                        }
                    }
                });

                || ()
            },
            (),
        );
        let s = succcess.then(|| Some(())).is_some();
        let html = html! {
            <div class="preview-tile">
                <p class="preview-name">{ iblob.name.clone() }</p>
                <p>{if s {HUNNIT} else {NOPE}}</p>
            </div>
        };
        result.push(html);
    }
    result.iter().map(VNode::to_owned).collect::<Html>()
}
