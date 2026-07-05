use iced::{Element, Length, Center, Theme};
use iced::widget::{column, row, text, center, pick_list, space, container, button, text_input, Column, combo_box};

use crate::{Message, MainLayout};
use crate::{APP_PADDING, APP_SPACING, IMAGE_SIZE_WIDTH, COMBOBOX_MENU_HEIGHT};
use crate::structs::{self, ConfigPages, WebPage};
use crate::log_err;

impl structs::AppPages {
	pub fn config_sidebar_view<'a>(&'a self, state: &'a MainLayout) -> Element<'a, Message> {
		let sidebar = container(
				column![
						text("Options:").size(22),
						button("General")
							.on_press_maybe(
								(state.config_curr_page != ConfigPages::AppSettings)
									.then_some(Message::ChangeConfigPage(ConfigPages::AppSettings)))
							.style(
								if state.config_curr_page == ConfigPages::AppSettings { button::danger }
								else { button::primary })
							.width(Length::Fill),
						button("Window")
							.on_press_maybe(
								(state.config_curr_page != ConfigPages::WindowSettings)
									.then_some(Message::ChangeConfigPage(ConfigPages::WindowSettings)))
							.style(
								if state.config_curr_page == ConfigPages::WindowSettings { button::danger }
								else { button::primary })
							.width(Length::Fill),
						button("Customisation")
							.on_press_maybe(
								(state.config_curr_page != ConfigPages::Customization)
									.then_some(Message::ChangeConfigPage(ConfigPages::Customization)))
							.style(
								if state.config_curr_page == ConfigPages::Customization { button::danger }
								else { button::primary })
							.width(Length::Fill),
						// button("Notifcations")
						// 	.on_press_maybe(
						// 		(state.config_curr_page != ConfigPages::PingSettings)
						// 			.then_some(Message::ChangeConfigPage(ConfigPages::PingSettings)))
						// 	.style(
						// 		if state.config_curr_page == ConfigPages::PingSettings { button::danger }
						// 		else { button::primary })
						// 	.width(Length::Fill),
						space::vertical(),
            button("Accept")
                .style(button::success)
							  .width(Length::Fill)
                .on_press_maybe(accept_press(state)),
            button("Discard")
                .style(button::danger)
							  .width(Length::Fill)
                .on_press_maybe(reject_press(state)),
						button("Credits")
							.on_press_maybe(
								(state.config_curr_page != ConfigPages::Credits)
									.then_some(Message::ChangeConfigPage(ConfigPages::Credits)))
							.style(
								if state.config_curr_page == ConfigPages::Credits { button::danger }
								else { button::primary })
							.width(Length::Fill),
					]
					.spacing(APP_SPACING)
					.padding(APP_PADDING)
			)
			.width(200)
			.max_width(200)
			.style(container::rounded_box);
		sidebar.into()
	}
	
	pub fn config_body_view<'a>(&self, state: &'a MainLayout) -> Element<'a, Message> {
		let body = match state.config_curr_page {
			ConfigPages::AppSettings => self.app_settings_body(state),
			ConfigPages::WindowSettings => self.window_settings_body(state),
			ConfigPages::Customization => self.customization_settings_body(state),
			ConfigPages::PingSettings => self.ping_settings_body(state),
			ConfigPages::Credits => self.credits_body(state),
		};
		
		let update_text = Some(text("New version available! To myself -> HIDE THIS"));
		
		let main = center(row![
				space::horizontal().width(Length::FillPortion(1)),
				body.width(Length::FillPortion(4)),
				space::horizontal().width(Length::FillPortion(1)),
			]);
		center(column![main, update_text].padding(APP_PADDING)).style(container::rounded_box).into()
	}
	
	fn app_settings_body<'a>(&self, state: &'a MainLayout) -> Column<'a, Message>  {
		let path = match crate::files::get_local_data_dir() {
			Ok(p) => format!("{}", p.into_os_string().display()),
			Err(err) => {
				log_err!["Error while getting save path: {}", err];
				String::from("Cannot get path. Check logs.")
			} 
		};
		column![
				row![
						text("Update interval:"),
						space::horizontal(),
						text_input("", &state.extra_string) // &state.config_settings.app_update_interval.to_string()
							.on_input(Message::ConfigChangeUpdateInterval)
							.width(Length::Fixed(100.0))
							.align_x(Center),
						text("sec"),
					].spacing(APP_SPACING)
					.align_y(iced::Center),
				row![
						text("Local data directory:"),
						space::horizontal(),
						button(text(path.clone())).on_press(Message::OpenExplorer(path)),
					].spacing(APP_SPACING),
				space::vertical().height(Length::Fixed(50.0)),
				row![
						space::horizontal(),
						button("Delete Item View data")
							.on_press(Message::ResetItemView),
						button("Delete Alchemy data")
							.on_press(Message::ResetAlchemy),
						button("Delete Calculator data")
							.on_press(Message::ResetCalculator),
						space::horizontal(),
					].spacing(APP_SPACING),
				space::vertical().height(Length::Fixed(10.0)),
				container(button("Delete all data")
					.on_press(Message::ResetAllData)
					.style(button::danger)).center_x(Length::Fill),
			]
			.spacing(APP_SPACING)
	}
	
	fn window_settings_body<'a>(&self, state: &'a MainLayout) -> Column<'a, Message> {
		let combo = combo_box(
        &state.config_window_combo_data,
        "Select window resolution",
        Some(&state.config_settings.new_resolution),
        Message::ConfigChangeResolutionNew,
        )
        .menu_height(Length::Fixed(COMBOBOX_MENU_HEIGHT))
			  .width(400);

    column![
				// text("Window size").width(Length::Fill).center(), // Change to dropbox
				// row![
				// 		space::horizontal(),
				// 		text_input("Width", &state.extra_string_1)
				// 			.on_input(Message::ConfigChangeResolutionWidth)
				// 			.align_x(Center),
				// 		space::horizontal(),
				// 		text_input("Height", &state.extra_string_2)
				// 			.on_input(Message::ConfigChangeResolutionHeight)
				// 			.align_x(Center),
				// 		space::horizontal(),
				// 	].spacing(APP_SPACING),
        text("Select window resolution:"),
        combo,
        space::horizontal().width(Length::Fixed(50.0)),
    ]
			.spacing(APP_SPACING)
      .align_x(Center)
	}
	
	fn ping_settings_body<'a>(&self, _state: &'a MainLayout) -> Column<'a, Message> {
		column![
				text("Nothing to see here... for now"),
			]
			.spacing(APP_SPACING)
	}
	
	fn customization_settings_body<'a>(&self, state: &'a MainLayout) -> Column<'a, Message> {
		column![
		    pick_list(Theme::ALL, state.theme(), Message::ConfigChangeTheme,)
            .placeholder("Theme"),
      ]
			.spacing(APP_SPACING)
	}
	
	fn credits_body<'a>(&self, _state: &'a MainLayout) -> Column<'a, Message> {
		let c = column![
        text("Where you can find me").size(24).width(Length::Fill).center(),
        row![
            space::horizontal(),
            button(iced::widget::image("img/itch.png").width(Length::Fixed(IMAGE_SIZE_WIDTH)))
              .on_press(Message::OpenWebPage(WebPage::Itch))
              .style(button::text),
            button(iced::widget::image("img/twitter.png").width(Length::Fixed(IMAGE_SIZE_WIDTH)))
              .on_press(Message::OpenWebPage(WebPage::Twitter))
              .style(button::text),
            button(iced::widget::image("img/github.png").width(Length::Fixed(IMAGE_SIZE_WIDTH)))
              .on_press(Message::OpenWebPage(WebPage::Github))
              .style(button::text),
            space::horizontal(),
          ].spacing(APP_SPACING),
        space::horizontal().width(Length::Fixed(20.0)),
        text("Thank you for using my silly little tool, I hope it was useful for at least a minute or two :P.").width(Length::Fill).center(),
        text("This calculator will be updated and new functions will be added. If you have an idea how to improve this app, write an issue or send me a DM.")
            .width(Length::Fill).center(),
      ]
			.spacing(APP_SPACING);
	  c
  }

}

fn accept_press(state: &'_ MainLayout) -> Option<Message> {
  if state.is_config_changed {
      Some(Message::AcceptNewSettings)
  }
  else {
      None
  }
}

fn reject_press(state: &'_ MainLayout) -> Option<Message>{
  if state.is_config_changed {
      Some(Message::RejectNewSettings)
  }
  else {
      None
  }
}
