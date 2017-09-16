use std::str::Chars;

extern crate i3ipc;
use i3ipc::{I3Connection, I3EventListener, Subscription};
use i3ipc::event::Event;
use i3ipc::event::inner::WindowChange;
use i3ipc::reply::{Node, NodeType};

static MARKS: &'static str = "QWERTYUIOP";

fn refresh_all_marks(conn: &mut I3Connection) {
    let ws = conn.get_workspaces().unwrap();
    let visible_ws = ws.workspaces.into_iter().filter(|w| w.visible).map(|w| w.name).collect();

    let tree = conn.get_tree().unwrap();
    mark_windows(MARKS.chars(), &tree, &visible_ws, conn);
}

fn mark_window(mark: char, id: i64, conn: &mut I3Connection) {
    let mark = format!(" {} ", mark);
    let cmd = format!("[con_id=\"{}\"] mark --replace {:?}", id, mark);
    conn.command(&cmd).unwrap();
}

fn mark_windows<'a>(mut marks: Chars<'a>, node: &Node, visible_ws: &Vec<String>, conn: &mut I3Connection) -> Chars<'a> {
    if node.window.is_some() {
        match marks.next() {
            Some(m) => mark_window(m, node.id, conn),
            None => return marks,
        }
    }

    match node.nodetype {
        NodeType::DockArea => return marks,
        NodeType::Workspace => {
            if let Some(ref name) = node.name {
                if visible_ws.iter().position(|x| *x == *name).is_none() {
                    return marks
                }
            }
        },
        _ => (),
    }

    for child in node.nodes.iter() {
        marks = mark_windows(marks, child, visible_ws, conn);
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
