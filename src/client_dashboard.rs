
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
                    draw_realm(t, &chunks[1], &data);
                }
			}
        });
	// end Group::default()

    terminal.draw()
}

fn draw_header(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {
	Group::default()
        .direction(Direction::Horizontal)
		.sizes(&[Size::Percent(35), Size::Percent(65)])
        .render(t, area, |t, chunks| {
        	Paragraph::default()
		        .text(
		            "cursor {mod=bold ↑→↓←}\nexit {mod=bold q}",
		        ).block(Block::default().title("Abstract").borders(Borders::ALL))
		        .render(t, &chunks[0]);
    		// end Paragraph::default()

        	Paragraph::default()
		        .text(
		            &format!("id {{mod=bold {}}}", data.id),
		        ).block(Block::default().title("Client").borders(Borders::ALL))
                .wrap(true)
		        .render(t, &chunks[1]);
    		// end Paragraph::default()
    	});
	// end Group::default()
}

fn draw_realms_list(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {
	Group::default()
        .direction(Direction::Vertical)
		.sizes(&[Size::Min(0)])
        .render(t, area, |t, chunks| {

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
                .render(t, &chunks[0]);
    		// end SelectableList::default()
        });
    // end Group::default()
}

fn draw_realm(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {

	Group::default()
        .direction(Direction::Vertical)
		.sizes(&[Size::Min(0)])
        .render(t, area, |t, chunks| {
        	draw_realm_ui(t, &chunks[0], &data);

        });
    // end Group::default()
}


fn draw_realm_ui(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {
	Group::default()
	    .direction(Direction::Horizontal)
		.sizes(&[Size::Fixed(16), Size::Min(0)])
	    .render(t, area, |t, chunks| {

			let region_index = data.realm.island.regions.current_index();
			let mut regions: Vec<String> = data.realm.island.regions.iter().map(|region| {
				format!("{}", region.1)
		    }).collect();

            regions.sort();

	    	let mut border_style = Style::default();
	    	if let InteractiveUi::Regions = data.active {
	    	    border_style = Style::default().fg(Color::Yellow);
	    	}
	        SelectableList::default()
	            .block(Block::default().borders(Borders::ALL).title(
                    &format!("Island {}", data.realm.id.clone())
                )
                .border_style(border_style))
	            .items(&regions)
	            .select(region_index)
	            .highlight_style(
	                Style::default().fg(Color::Yellow)
	            )
	            .render(t, &chunks[0]);
    		// end SelectableList::default()

			Group::default()
		        .direction(Direction::Vertical)
	    		.sizes(&[Size::Fixed(8), Size::Fixed(10), Size::Min(0)])
		        .render(t, &chunks[1], |t, chunks| {
					draw_realm_expedition(t, &chunks[0], &data);
					if let InteractiveUi::Explorers = data.active {
		        		if data.realm.expedition.explorers.current().expect("could not fetch current explorers selection.").region.is_some() {
							draw_realm_region(t, &chunks[1], &data);
		        		} else {
					    	Paragraph::default()
						        .text(
						            "this explorer has not embarked yet. select the embark order and move them to a region."
						        ).block(Block::default().borders(Borders::ALL).title("Region").border_style(Style::default()))
								.wrap(true)
						        .render(t, &chunks[1]);
							// end Paragraph::default()	
			        	}
		        	} else {
						draw_realm_region(t, &chunks[1], &data);
		        	}

                    let style = Style::default();
                    let done = Style::default().fg(Color::Green);
                    let mut border_style = Style::default();
                    if data.realm.done {
                        border_style = Style::default().fg(Color::Green);
                    }

                    let objectives = data.realm.objectives.iter().map(|objective| {
                        if data.realm.completed.contains(objective) {
                            Item::StyledData(
                                format!("{}", objective),
                                &done
                            )
                        } else {
                            Item::StyledData(
                                format!("{}", objective),
                                &style
                            )
                        }
                    });

                    List::new(objectives)
                        .block(Block::default()
                            .title(&data.realm.title)
                            .borders(Borders::ALL)
                            .border_style(border_style))
                        .render(t, &chunks[2]);
                    // end List::new()
	        	});
	        // end Group::default()
		});
	// end Group::default()
}

fn draw_realm_region(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {

    let region = data.realm.island.regions.current().expect("could not fetch current regions selection.");

    Group::default()
        .direction(Direction::Horizontal)
        .sizes(&[Size::Percent(50), Size::Percent(50)])
        .render(t, area, |t, chunks| {

            let style = Style::default();
            let cyan = Style::default().fg(Color::Cyan);
            let green = Style::default().fg(Color::Green);

            let mut info = vec![];
            info.push(Item::StyledData(
                    format!("{:?}", region.buildings.storage()),
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
            for explorer in data.realm.expedition.explorers.iter() {
                if let Some(explorer_region) = explorer.region {
                    if explorer_region == region.id {
                        info.push(Item::StyledData(
                                format!("{:?}", explorer.traits.storage()),
                                &cyan
                        ));
                    }
                }
            }

            List::new(info.into_iter())
                .block(Block::default().borders(Borders::ALL).title(&format!("{} {}", "Region", region)))
                .render(t, &chunks[0]);
            // end List::new()

            let particularities_index = region.particularities.current_index();

            let particularities: Vec<String> = region.particularities.iter().map(|particularity| {
                format!("{:?}", particularity)
            }).collect();

            match data.active {
                InteractiveUi::Particularities => {
                    SelectableList::default()
                        .block(Block::default().borders(Borders::ALL).title("Particularities [Enter pick/investigate]")
                        .border_style(Style::default().fg(Color::Yellow)))
                        .items(&particularities)
                        .select(particularities_index)
                        .highlight_style(
                            Style::default().fg(Color::Yellow)
                        )
                        .highlight_symbol("→")
                        .render(t, &chunks[1]);
                    // end SelectableList::default()
                },
                _ => {
                    SelectableList::default()
                        .block(Block::default().borders(Borders::ALL).title("Particularities")
                        .border_style(Style::default()))
                        .items(&particularities)
                        .select(particularities_index)
                        .render(t, &chunks[1]);
                    // end SelectableList::default()
                }
            }

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

    let explorer_index = data.realm.expedition.explorers.current_index();
    let explorers: Vec<String> = data.realm.expedition.explorers.iter().map(|explorer| {
        if let Some(explorer_region) = explorer.region {
            format!("{:?} at {}", explorer.traits.storage(), explorer_region)
        } else {
            format!("{:?}", explorer.traits.storage())
        }
    }).collect();

    match data.active {
        InteractiveUi::Explorers => {
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Expedition [Enter]").border_style(Style::default().fg(Color::Yellow)))
                .items(&explorers)
                .select(explorer_index)
                .highlight_style(
                    Style::default().fg(Color::Yellow)
                ).highlight_symbol("→")
                .render(t, area);
            // end SelectableList::default()
        },
        InteractiveUi::ExplorerMove | InteractiveUi::ExplorerActions | InteractiveUi::ExplorerInventory | InteractiveUi::ExplorerOrders => {
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Expedition").border_style(Style::default()))
                .items(&explorers)
                .select(explorer_index)
                .highlight_style(
                    Style::default().fg(Color::Yellow)
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
                    Style::default().fg(Color::Yellow)
                )
                .render(t, area);
            // end SelectableList::default()
        }
    }
}

fn draw_realm_expedition_explorer(t: &mut Terminal<RawBackend>, area: &Rect, data: &Data) {

    let region_index = data.realm.island.regions.current_index();
    let regions: Vec<String> = data.realm.island.regions.iter().map(|region| {
        format!("{}", region.1)
    }).collect();

    let explorer_orders_index = data.explorer_orders.current_index();
    let explorer_orders: Vec<String> = data.explorer_orders.iter().map(|explorer_order| {
        format!("{:?}", explorer_order)
    }).collect();


    let mut inventory_index = 0;
    let mut inventory: Vec<String> = vec![];
    if let Some(explorer) = data.realm.expedition.explorers.current() {
        inventory_index = explorer.inventory.current_index();
        inventory = explorer.inventory.iter().map(|item| {
            format!("{:?}", item)
        }).collect();
    }

    match data.active {
        InteractiveUi::Explorers => {
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Explorer").border_style(Style::default()))
                .items(&explorer_orders)
                .select(explorer_orders_index)
                .highlight_style(
                    Style::default()
                )
                .render(t, area);
            // end SelectableList::default()
        },
        InteractiveUi::ExplorerOrders => {
            SelectableList::default()
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Explorer [Enter]")
                    .border_style(Style::default().fg(Color::Yellow)))
                .items(&explorer_orders)
                .select(explorer_orders_index)
                .highlight_style(
                    Style::default().fg(Color::Yellow)
                )
                .highlight_symbol("→")
                .render(t, area);
            // end SelectableList::default()
        },
        InteractiveUi::ExplorerInventory => {
            SelectableList::default()
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Inventory [Bsp to exit, Enter drop/forget]")
                    .border_style(Style::default().fg(Color::Yellow))
                    .title_style(Style::default().fg(Color::Yellow)))
                .items(&inventory)
                .select(inventory_index)
                .highlight_style(
                    Style::default().fg(Color::Yellow)
                )
                .highlight_symbol("→")
                .render(t, area);
            // end SelectableList::default()
        },
        InteractiveUi::ExplorerActions => {
            if let Some(explorer) = data.realm.expedition.explorers.current() {
                Paragraph::default()
                    .text(&format!("{:?}", explorer.trait_actions()))
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title("Actions [Bsp to exit]")
                        .border_style(Style::default().fg(Color::Yellow))
                        .title_style(Style::default().fg(Color::Yellow)))
                    .wrap(true)
                    .render(t, area);
                // end Paragraph::default() 
            }
        },
        InteractiveUi::ExplorerMove => {
            let mut title = "Move [Bsp to exit]".to_string();
            if let Some(explorer) = data.realm.expedition.explorers.current() {
                if !explorer.region.is_some() {
                    title = "Embark [Bsp to exit]".to_string();
                }
            }
            SelectableList::default()
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(&title)
                    .border_style(Style::default().fg(Color::Yellow))
                    .title_style(Style::default().fg(Color::Yellow)))
                .items(&regions)
                .select(region_index)
                .highlight_style(
                    Style::default().fg(Color::Yellow)
                )
                .highlight_symbol("→")
                .render(t, area);
            // end SelectableList::default()
        },
        _ => {
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("Explorer").border_style(Style::default()))
                .items(&explorer_orders)
                .select(explorer_orders_index)
                .render(t, area);
            // end SelectableList::default()
        },
    }
}