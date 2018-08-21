
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
		.sizes(&[Size::Fixed(4), Size::Min(0)])
        .render(terminal, &terminal_area, |t, chunks| {

        	draw_header(t, &chunks[0], &data);

	        match data.active {
			    InteractiveUi::Realms => {
    				draw_realms_list(t, &chunks[1], &data);
			    },
                _ => {
                    if data.realm.is_some() {
                        draw_realm(t, &chunks[1], &data);
                    }
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
		            "move cursor with {mod=bold ↑→↓←}\nexit with {mod=bold q}",
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

fn draw_realm_info(t: &mut Terminal<RawBackend>, area: &Rect, _data: &Data) {
    Paragraph::default()
        .text(
            "switch to realms list with {mod=bold l}"
        ).block(Block::default())
        .render(t, area);
    // end Paragraph::default()
}


fn draw_realm_ui(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {
	Group::default()
	    .direction(Direction::Horizontal)
		.sizes(&[Size::Fixed(16), Size::Min(0)])
	    .render(t, area, |t, chunks| {

			let region_index = data.regions.current_index();
			let regions: Vec<String> = data.regions.iter().map(|region| {
				format!("{}", region)
		    }).collect();

	    	let mut border_style = Style::default();
	    	if let InteractiveUi::Regions = data.active {
	    	    border_style = Style::default().fg(Color::Yellow);
	    	}
	    	let mut regions_list_style = Style::default();
	    	if let InteractiveUi::Regions = data.active {
	    	    regions_list_style = Style::default().fg(Color::Yellow);
	    	}
            let mut island_title = "Island".to_string();
            if let Some(ref realm) = data.realm {
                island_title = format!("Island {}", realm.id.clone());
            }
	        SelectableList::default()
	            .block(Block::default().borders(Borders::ALL).title(
                    &island_title
                )
                .border_style(border_style))
	            .items(&regions)
	            .select(region_index)
	            .highlight_style(
	                regions_list_style
	            )
	            .render(t, &chunks[0]);
    		// end SelectableList::default()

			Group::default()
		        .direction(Direction::Vertical)
	    		.sizes(&[Size::Fixed(8), Size::Fixed(10), Size::Min(0)])
		        .render(t, &chunks[1], |t, chunks| {
					draw_realm_expedition(t, &chunks[0], &data);
					if let InteractiveUi::Explorers = data.active {
		        		if data.explorers.current().expect("could not fetch current explorers selection.").region.is_some() {
							draw_realm_region(t, &chunks[1], &data);
		        		} else {
					    	Paragraph::default()
						        .text(
						            "this explorer has not embarked yet. select a region from the list to move them there."
						        ).block(Block::default().borders(Borders::ALL).title("Region").border_style(Style::default()))
								.wrap(true)
						        .render(t, &chunks[1]);
							// end Paragraph::default()	
			        	}
		        	} else {
						draw_realm_region(t, &chunks[1], &data);
		        	}

                    Paragraph::default()
                        .text(
                            "placeholder"
                        ).block(Block::default().title("Briefing").borders(Borders::ALL).border_style(Style::default().fg(Color::Gray)))
                        .wrap(true)
                        .render(t, &chunks[2]);
                    // end Paragraph::default() 
	        	});
	        // end Group::default()
		});
	// end Group::default()
}

fn draw_realm_region(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {

    let region = data.regions.current().expect("could not fetch current regions selection.");

    Group::default()
        .direction(Direction::Horizontal)
        .sizes(&[Size::Percent(50), Size::Percent(50)])
        .render(t, area, |t, chunks| {

            let style = Style::default();
            let cyan = Style::default().fg(Color::Cyan);
            let green = Style::default().fg(Color::Green);

            let mut info = vec![];
            info.push(Item::StyledData(
                    format!("{:?}", region.buildings),
                    &style
            ));
            info.push(Item::StyledData(
                    format!("Resources {}", region.resources),
                    &style
            ));
            if region.mapped {
                info.push(Item::StyledData(
                        format!("Mapped"),
                        &green
                ));
            }
            for explorer in data.explorers.iter() {
                if let Some(explorer_region) = explorer.region {
                    if explorer_region == region.id {
                        info.push(Item::StyledData(
                                format!("{:?}", explorer.traits),
                                &cyan
                        ));
                    }
                }
            }

            List::new(info.into_iter())
                .block(Block::default().borders(Borders::ALL).title(&format!("{} {}", "Region", region)))
                .render(t, &chunks[0]);
            // end List::new()

            let particularities = region.particularities.iter().map(|particularity| {
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

fn draw_realm_expedition(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {
	Group::default()
        .direction(Direction::Horizontal)
		.sizes(&[Size::Percent(50), Size::Percent(50)])
        .render(t, area, |t, chunks| {
            draw_realm_expedition_list(t, &chunks[0], &data);

            draw_realm_expedition_explorer(t, &chunks[1], &data);
        });
	// end Group::default()
}

fn draw_realm_expedition_list(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {

    let explorer_index = data.explorers.current_index();
    let explorers: Vec<String> = data.explorers.iter().map(|explorer| {
        if let Some(explorer_region) = explorer.region {
            format!("{:?} {}", explorer.traits, explorer_region)
        } else {
            format!("{:?}", explorer.traits)
        }
    }).collect();

    match data.active {
        InteractiveUi::Explorers => {
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Expedition [Enter]").border_style(Style::default().fg(Color::Yellow)))
                .items(&explorers)
                .select(explorer_index)
                .highlight_style(
                    Style::default().fg(Color::Yellow),
                ).highlight_symbol("→")
                .render(t, area);
            // end SelectableList::default()
        },
        InteractiveUi::ExplorerMove | InteractiveUi::ExplorerActions | InteractiveUi::ExplorerInventory | InteractiveUi::ExplorerSelect => {
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Expedition").border_style(Style::default().fg(Color::Yellow)))
                .items(&explorers)
                .select(explorer_index)
                .highlight_style(
                    Style::default().fg(Color::Yellow),
                )
                .render(t, area);
            // end SelectableList::default()
        },
        _ => {
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Expedition").border_style(Style::default()))
                .items(&explorers)
                .select(explorer_index)
                .highlight_style(
                    Style::default()
                )
                .render(t, area);
            // end SelectableList::default()
        }
    }
}

fn draw_realm_expedition_explorer(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {

    let region_index = data.regions.current_index();
    let regions: Vec<String> = data.regions.iter().map(|region| {
        format!("{}", region)
    }).collect();

    let explorer_select_index = data.explorer_select.current_index();
    let explorer_select: Vec<String> = data.explorer_select.iter().map(|explorer_select| {
        format!("{:?}", explorer_select)
    }).collect();

    match data.active {
        InteractiveUi::ExplorerMove => {
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Move Explorer [Esc]").border_style(Style::default().fg(Color::Yellow)))
                .items(&regions)
                .select(region_index)
                .highlight_style(
                    Style::default().fg(Color::Yellow)
                )
                .highlight_symbol("→")
                .render(t, area);
            // end SelectableList::default()
        },
        InteractiveUi::ExplorerActions => {
            if let Some(explorer) = data.explorers.current() {
                Paragraph::default()
                    .text(
                        &format!("{:?}", explorer.actions())
                    ).block(Block::default().borders(Borders::ALL).title("Explorer Actions [Esc]").border_style(Style::default().fg(Color::Yellow)))
                    .wrap(true)
                    .render(t, area);
                // end Paragraph::default() 
            }
        },
        InteractiveUi::ExplorerInventory => {
            if let Some(explorer) = data.explorers.current() {
                Paragraph::default()
                    .text(
                        &format!("{:?}", explorer.inventory)
                    ).block(Block::default().borders(Borders::ALL).title("Explorer Inventory [Esc]").border_style(Style::default().fg(Color::Yellow)))
                    .wrap(true)
                    .render(t, area);
                // end Paragraph::default() 
            }
        },
        InteractiveUi::Explorers => {
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Explorer").border_style(Style::default()))
                .items(&explorer_select)
                .select(explorer_select_index)
                .highlight_style(
                    Style::default()
                )
                .render(t, area);
            // end SelectableList::default()
        },
        InteractiveUi::ExplorerSelect => {
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Explorer [Enter]").border_style(Style::default().fg(Color::Yellow)))
                .items(&explorer_select)
                .select(explorer_select_index)
                .highlight_style(
                    Style::default().fg(Color::Yellow)
                )
                .highlight_symbol("→")
                .render(t, area);
            // end SelectableList::default()
        },
        _ => {
            if let Some(_) = data.explorers.current() {
                Paragraph::default()
                    .text(
                        ""
                    ).block(Block::default().borders(Borders::ALL).title("Explorer").border_style(Style::default()))
                    .wrap(true)
                    .render(t, area);
                // end Paragraph::default() 
            } else {
                Paragraph::default()
                    .text(
                        "select an explorer from the expedition to give orders."
                    ).block(Block::default().borders(Borders::ALL).title("Explorer").border_style(Style::default()))
                    .wrap(true)
                    .render(t, area);
                // end Paragraph::default() 
            }
        },
    }
}