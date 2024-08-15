use std::sync::Mutex;

use anathema::widgets::components::events::Event;
use anathema::{backend::Backend, prelude::TuiBackend};

pub static SHOULD_QUIT: Mutex<bool> = Mutex::new(false);

pub struct QuittableTuiBackend (pub TuiBackend);

impl Backend for QuittableTuiBackend {
    fn size(&self) -> anathema::geometry::Size {
        self.0.size()
    }

    fn quit_test(&self, event: Event) -> bool {
        self.0.quit_test(event) || SHOULD_QUIT.try_lock().map(|el| *el).unwrap_or(false)
    }

    fn next_event(&mut self, timeout: std::time::Duration) -> Option<Event> {
        let Ok(mut should_quit) = SHOULD_QUIT.try_lock() else { return Some(Event::Stop); };
        if *should_quit {
            *should_quit = false;
            return Some(Event::Stop);
        }
        self.0.next_event(timeout)
    }

    fn resize(&mut self, new_size: anathema::geometry::Size) {
        self.0.resize(new_size);
    }

    fn paint<'bp>(
        &mut self,
        element: &mut anathema::widgets::Element<'bp>,
        children: &[anathema::store::tree::Node],
        values: &mut anathema::store::tree::TreeValues<anathema::widgets::WidgetKind<'bp>>,
        text: &mut anathema::widgets::layout::text::StringSession<'_>,
        attribute_storage: &anathema::widgets::AttributeStorage<'bp>,
        ignore_floats: bool,
    ) {
        self.0.paint(
            element,
            children,
            values,
            text,
            attribute_storage,
            ignore_floats,
        );
    }

    fn render(&mut self) {
        self.0.render()
    }

    fn clear(&mut self) {
        self.0.clear();
    }
}
