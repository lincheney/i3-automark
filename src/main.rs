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
    mark_windows(0, &tree, &visible_ws, conn);
}

fn mark_window(ix: usize, id: i64, conn: &mut I3Connection) {
    let mark = format!(" {} ", &MARKS[ix..ix+1]);
    let cmd = format!("[con_id=\"{}\"] mark --replace {:?}", id, mark);
    conn.command(&cmd).unwrap();
}

fn mark_windows(ix: usize, node: &Node, visible_ws: &Vec<String>, conn: &mut I3Connection) -> usize {
    let mut ix = ix;
    if node.window.is_some() {
        mark_window(ix, node.id, conn);
        ix += 1;
    }

    match node.nodetype {
        NodeType::DockArea => return ix,
        NodeType::Workspace => {
            if let Some(ref name) = node.name {
                if visible_ws.iter().position(|x| *x == *name).is_none() {
                    return ix;
                }
            }
        },
        _ => (),
    }

    for child in node.nodes.iter() {
        ix = mark_windows(ix, child, visible_ws, conn);
    }
    ix
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
