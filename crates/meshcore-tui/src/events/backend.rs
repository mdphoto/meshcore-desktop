use crate::action::Action;
use meshcore_service::AppEvent;

pub fn from_app_event(event: AppEvent) -> Action {
    Action::Backend(event)
}
