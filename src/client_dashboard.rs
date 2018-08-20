
use std::io;

use tui::Terminal;
use tui::backend::RawBackend;
use tui::layout::{Direction, Group, Size, Rect};
use tui::widgets::{Widget, Paragraph, Block, Borders, List, Item, SelectableList};
use tui::style::{Style, Color};

use client::*;

pub fn draw(terminal: &mut Terminal<RawBackend>, data: &mut Data) -> Result<(), io::Error> {
	let terminal_area = terminal.size().expect("could not get terminal size.");
			
	Group::default()
        .direction(Direction::Vertical)
		.sizes(&[Size::Fixed(6), Size::Min(0)])
        .render(terminal, &terminal_area, |t, chunks| {

        	draw_header(t, &chunks[0], &data);

	        match data.active {
			    InteractiveUi::Locations | InteractiveUi::Explorers | InteractiveUi::MoveLocations => {
			    	if data.realm.is_some() {
			    		draw_realm(t, &chunks[1], &data);
			        }
			    },
			    InteractiveUi::Realms => {
    				draw_realms_list(t, &chunks[1], &data);
			    }
			}
        });
	// end Group::default()

    terminal.draw()
}

fn draw_header(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {
	Group::default()
        .direction(Direction::Horizontal)
		.sizes(&[Size::Percent(75), Size::Percent(25)])
        .render(t, area, |t, chunks| {
        	Paragraph::default()
		        .text(
		            "move cursor with {mod=bold ↓↑}\nswitch with {mod=bold → ←}\npick with {mod=bold Enter}\nexit with {mod=bold q}",
		        ).block(Block::default().title("Abstract").borders(Borders::ALL))
		        .render(t, &chunks[0]);
    		// end Paragraph::default()

        	Paragraph::default()
		        .text(
		            &format!("id {{mod=bold {}}}", data.id),
		        ).block(Block::default().title("Client").borders(Borders::ALL))
		        .render(t, &chunks[1]);
    		// end Paragraph::default()
    	});
	// end Group::default()
}

fn draw_realms_list(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {
	Group::default()
        .direction(Direction::Vertical)
		.sizes(&[Size::Fixed(2), Size::Min(0)])
        .render(t, area, |t, chunks| {
        	Paragraph::default()
		        .text(
		            "request new realm with {mod=bold r}"
		        ).block(Block::default())
		        .render(t, &chunks[0]);
    		// end Paragraph::default()

        	let border_style = Style::default().fg(Color::Yellow);

        	let realms_index = data.realms.current_index();
        	let realms: Vec<String> = data.realms.iter().map(|realm| {
                format!("{}", realm)
            }).collect();

            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Realms")
            	.border_style(border_style))
                .items(&realms)
                .select(realms_index)
                .highlight_style(
                    Style::default().fg(Color::Yellow),
                )
                .highlight_symbol("→")
                .render(t, &chunks[1]);
    		// end SelectableList::default()
        });
    // end Group::default()
}

fn draw_realm(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {

	Group::default()
        .direction(Direction::Vertical)
		.sizes(&[Size::Fixed(2), Size::Min(0)])
        .render(t, area, |t, chunks| {

        	draw_realm_info(t, &chunks[0], &data);

        	draw_realm_ui(t, &chunks[1], &data);

        });
    // end Group::default()
}

fn draw_realm_info(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {
	if let Some(ref realm) = data.realm {
    	Paragraph::default()
	        .text(
	            &format!("current realm {{mod=bold {}}}; switch to realms list with {{mod=bold l}}", realm.id)
	        ).block(Block::default())
	        .render(t, area);
		// end Paragraph::default()
	} else {
    	Paragraph::default()
	        .text(
	            "switch to realms list with {mod=bold l}"
	        ).block(Block::default())
	        .render(t, area);
		// end Paragraph::default()
	}
}


fn draw_realm_ui(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {
	Group::default()
	    .direction(Direction::Horizontal)
		.sizes(&[Size::Fixed(16), Size::Min(0)])
	    .render(t, area, |t, chunks| {

			let location_index = data.locations.current_index();
			let locations: Vec<String> = data.locations.iter().map(|tile| {
				format!("{}", tile)
		    }).collect();

	    	let mut border_style = Style::default();
	    	if let InteractiveUi::Locations = data.active {
	    	    border_style = Style::default().fg(Color::Yellow);
	    	}
	    	let mut locations_list_style = Style::default();
	    	if let InteractiveUi::Locations = data.active {
	    	    locations_list_style = Style::default().fg(Color::Yellow);
	    	}
	        SelectableList::default()
	            .block(Block::default().borders(Borders::ALL).title("Island").border_style(border_style))
	            .items(&locations)
	            .select(location_index)
	            .highlight_style(
	                locations_list_style
	            )
	            .render(t, &chunks[0]);
    		// end SelectableList::default()

			Group::default()
		        .direction(Direction::Vertical)
	    		.sizes(&[Size::Fixed(8), Size::Min(0)])
		        .render(t, &chunks[1], |t, chunks| {
					draw_realm_expedition(t, &chunks[0], &data);
					if let InteractiveUi::Explorers = data.active {
		        		if data.explorers.current().expect("could not fetch current explorers selection.").location.is_some() {
							draw_realm_location(t, &chunks[1], &data);
		        		} else {
					    	Paragraph::default()
						        .text(
						            "this explorer has not embarked yet. select a location from the list to move them there."
						        ).block(Block::default().borders(Borders::ALL).title("Location").border_style(Style::default()))
								.wrap(true)
						        .render(t, &chunks[1]);
							// end Paragraph::default()	
			        	}
		        	} else {
						draw_realm_location(t, &chunks[1], &data);
		        	}
	        	});
	        // end Group::default()
		});
	// end Group::default()
}

fn draw_realm_expedition(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {

	let explorer_index = data.explorers.current_index();
	let explorers: Vec<String> = data.explorers.iter().map(|explorer| {
		if let Some(explorer_location) = explorer.location {
        	format!("{} {}", explorer.variant, explorer_location)
		} else {
        	format!("{}", explorer.variant)
		}
    }).collect();

	let location_index = data.locations.current_index();
	let locations: Vec<String> = data.locations.iter().map(|tile| {
		format!("{}", tile)
    }).collect();

	Group::default()
        .direction(Direction::Horizontal)
		.sizes(&[Size::Percent(50), Size::Percent(50)])
        .render(t, area, |t, chunks| {
        	if let InteractiveUi::Explorers = data.active {
                SelectableList::default()
                    .block(Block::default().borders(Borders::ALL).title("Expedition").border_style(Style::default().fg(Color::Yellow)))
                    .items(&explorers)
                    .select(explorer_index)
                    .highlight_style(
                        Style::default().fg(Color::Yellow),
                    ).highlight_symbol("→")
                    .render(t, &chunks[0]);
	        	// end SelectableList::default()
        	} else if let InteractiveUi::MoveLocations = data.active {
                SelectableList::default()
                    .block(Block::default().borders(Borders::ALL).title("Expedition").border_style(Style::default().fg(Color::Yellow)))
                    .items(&explorers)
                    .select(explorer_index)
                    .highlight_style(
                        Style::default().fg(Color::Yellow),
                    )
                    .render(t, &chunks[0]);
	        	// end SelectableList::default()
        	} else {
                SelectableList::default()
                    .block(Block::default().borders(Borders::ALL).title("Expedition").border_style(Style::default()))
                    .items(&explorers)
                    .select(explorer_index)
                    .highlight_style(
                        Style::default()
                    )
                    .render(t, &chunks[0]);
	        	// end SelectableList::default()
        	}

        	if let InteractiveUi::MoveLocations = data.active {
                SelectableList::default()
                    .block(Block::default().borders(Borders::ALL).title("Move Explorer").border_style(Style::default().fg(Color::Yellow)))
                    .items(&locations)
                    .select(location_index)
                    .highlight_style(
                        Style::default().fg(Color::Yellow)
                    )
                    .highlight_symbol("→")
                    .render(t, &chunks[1]);
	        	// end SelectableList::default()
        	} else if let InteractiveUi::Explorers = data.active {
        		if data.explorers.current().expect("could not fetch current explorers selection.").location.is_some() {
                    SelectableList::default()
                        .block(Block::default().borders(Borders::ALL).title("Move Explorer").border_style(Style::default()))
                        .items(&locations)
                        .select(location_index)
                        .highlight_style(
                            Style::default().fg(Color::Yellow)
                        )
                        .render(t, &chunks[1]);
		        	// end SelectableList::default()
        		} else {
                    SelectableList::default()
                        .block(Block::default().borders(Borders::ALL).title("Move Explorer").border_style(Style::default()))
                        .items(&locations)
                        .select(location_index)
                        .highlight_style(
                            Style::default()
                        )
                        .render(t, &chunks[1]);
		        	// end SelectableList::default()
        		}
        	} else {
		    	Paragraph::default()
			        .text(
			            "select an explorer from the expedition to move them and make actions."
			        ).block(Block::default().borders(Borders::ALL).title("Move Explorer").border_style(Style::default()))
					.wrap(true)
			        .render(t, &chunks[1]);
				// end Paragraph::default()	
        	}

        });
	// end Group::default()
}

fn draw_realm_location(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {

	let location = data.locations.current().expect("could not fetch current locations selection.");

	Group::default()
        .direction(Direction::Horizontal)
		.sizes(&[Size::Percent(50), Size::Percent(50)])
        .render(t, area, |t, chunks| {

	        let style = Style::default();
        	let cyan = Style::default().fg(Color::Cyan);
        	let green = Style::default().fg(Color::Green);

			let mut info = vec![];
			info.push(Item::StyledData(
                    format!("{:?}", location.buildings),
                    &style
            ));
			info.push(Item::StyledData(
                    format!("Resources {}", location.resources),
                    &style
            ));
            if location.mapped {
    			info.push(Item::StyledData(
	                    format!("Mapped"),
	                    &green
                ));
            }
            for explorer in data.explorers.iter() {
            	if let Some(explorer_location) = explorer.location {
            		if explorer_location == location.id {
	        			info.push(Item::StyledData(
			                    format!("{}", explorer.variant),
			                    &cyan
		                ));
            		}
            	}
            }

    		List::new(info.into_iter())
                .block(Block::default().borders(Borders::ALL).title(&format!("{} {}", "Location", location)))
                .render(t, &chunks[0]);
    		// end List::new()

        	let particularities = location.particularities.iter().map(|particularity| {
                Item::StyledData(
                    format!("{:?}", particularity),
                    &style
                )
            });
    		List::new(particularities)
                .block(Block::default().borders(Borders::ALL).title(&format!("Particularities")))
                .render(t, &chunks[1]);
    		// end List::new()

        });
	// end Group::default()
}