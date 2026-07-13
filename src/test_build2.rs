use gtk::prelude::*; fn test(win: &gtk::ApplicationWindow) { let _ = win.focus_widget(); let _ = gtk::prelude::RootExt::focus(win); }
