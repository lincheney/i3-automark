use std::str::Chars;

extern crate i3ipc;
use i3ipc::{I3Connection, I3EventListener, Subscription};
use i3ipc::event::Event;
use i3ipc::event::inner::WindowChange;
use i3ipc::reply::{Node, NodeType};

const MARKS: &str = "QWERTYUIOP";

fn refresh_all_marks(conn: &mut I3Connection) {
    let mut workspaces = conn.get_workspaces().unwrap().workspaces;
    // sort left to right, top to bottom
    workspaces.sort_by_key(|w| (w.rect.1, w.rect.0));

    let tree = conn.get_tree().unwrap();

    let mut marks = MARKS.chars();
    for ws_name in workspaces.into_iter().filter(|w| w.visible).map(|w| w.name) {
        marks = mark_windows_on_ws(marks, &tree, &ws_name, conn);
    }
}

fn mark_window(mark: char, id: i64, conn: &mut I3Connection) {
    let cmd = format!("[con_id=\"{}\"] mark --replace {}", id, mark);
    conn.command(&cmd).unwrap();
}

fn mark_windows_on_ws<'a>(mut marks: Chars<'a>, node: &Node, ws_name: &String, conn: &mut I3Connection) -> Chars<'a> {
    if node.window.is_some() {
        match marks.next() {
            Some(m) => mark_window(m, node.id, conn),
            None => return marks,
        }
    }

    match node.nodetype {
        NodeType::DockArea => return marks,
        NodeType::Workspace if node.name.as_ref() != Some(ws_name) => {
            return marks
        },
        _ => (),
    }

    for child in node.nodes.iter().chain(node.floating_nodes.iter()) {
        marks = mark_windows_on_ws(marks, child, ws_name, conn);
    }
    marks
}

fn main() {
    let mut conn = I3Connection::connect().unwrap();
    refresh_all_marks(&mut conn);

    let mut listener = I3EventListener::connect().unwrap();
    let subs = [Subscription::Workspace, Subscription::Output, Subscription::Window];
    listener.subscribe(&subs).unwrap();

    for event in listener.listen() {
        match event.unwrap() {
            Event::WorkspaceEvent(_) | Event::OutputEvent(_) => refresh_all_marks(&mut conn),
            Event::WindowEvent(e) => {
                match e.change {
                    WindowChange::New | WindowChange::Close | WindowChange::Move => refresh_all_marks(&mut conn),
                    _ => (),
                }
            },
            _ => unreachable!()
        }
    }
}
