use winit::event_loop::EventLoopProxy;
use notify::{Event, event::{AccessMode,AccessKind,EventKind}, Result as NotifyResult, RecommendedWatcher};
use anyhow::Result;
use super::UserEvent;

pub fn build(proxy: EventLoopProxy<UserEvent>) -> Result<RecommendedWatcher>{
    Ok(notify::immediate_watcher(move |event: NotifyResult<Event>|{
        let event = if let Ok(x) = event{
            x
        }else{
            return;
        };
        if let EventKind::Access(x) = event.kind{
            if let AccessKind::Close(x) = x{
                if let AccessMode::Write = x{
                    for p in event.paths{
                        proxy.send_event(UserEvent::Reload(p)).unwrap();
                    }
                }
            }
        }
    })?)
}
