use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow, Button, ListBox, ListBoxRow};

const APP_ID: &str = "org.gtk_rs.HelloWorld2";

fn main() -> glib::ExitCode {
    // Create a new application
    let app: Application = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    // Create a button with label
    let button: Button = Button::builder().label("Press me!").build();

    // Connect to "clicked" signal of `button`
    button.connect_clicked(|button| {
        // Set the label to "Hello World!" after the button has been clicked
        button.set_label("Hello World!");
    });

    // Create a main screen (using a ListBox)
    let main_screen: ListBox = ListBox::builder().build();
    let row = ListBoxRow::builder().child(&button).build();
    main_screen.append(&row);

    // Create a sidebar
    let sidebar: gtk::Box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    sidebar.set_size_request(200, -1);

    // Create a Paned widget to split the window into sidebar and main screen
    let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
    paned.set_start_child(Some(&sidebar));
    paned.set_end_child(Some(&main_screen));

    // Create a window
    let window: ApplicationWindow = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .child(&paned)
        .build();

    window.set_default_size(800, 600);

    // Present window
    window.present();
}
