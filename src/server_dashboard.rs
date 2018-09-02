
use uuid::Uuid;
use std::collections::HashMap;
use server::Client;
use std::io;


use tui::Terminal;
use tui::backend::RawBackend;

use chrono::{Local, DateTime};

use tui::layout::{Direction, Group, Size};
use tui::widgets::{Block, Borders, Widget, Table, Row};
use tui::style::{Style, Color};

use tokens::*;

pub fn draw(t: &mut Terminal<RawBackend>, requests: &Vec<(ClientId, RealmsProtocol, DateTime<Local>)>, clients: &HashMap<Uuid, Client>, realms: &Vec<Realm>) -> Result<(), io::Error> {
	let t_size = t.size().unwrap();

	Group::default()
        .direction(Direction::Vertical)
		.sizes(&[Size::Percent(40), Size::Percent(30), Size::Percent(30)])
        .render(t, &t_size, |t, chunks| {
            let style = Style::default();
            let highlight = Style::default().fg(Color::Yellow);
            let done = Style::default().fg(Color::Green);
            let quit = Style::default().fg(Color::Red);


        	let requests = requests.iter().rev().map(|(client_id, request, time)| {
        		let mut client_index = 0;
        		for (index, (_, client)) in &mut clients.iter().enumerate() {
	    		    if client.id == *client_id {
	    		    	client_index = index;
	    		    }
	    		}

        		match request {
                    RealmsProtocol::Register => {
                        Row::StyledData(vec![format!("{}", client_index), format!("{}", request), format!("{}", time.format("%H:%M:%S %d.%m.%y"))].into_iter(), &highlight)
                    },
                    RealmsProtocol::Quit => {
                        Row::StyledData(vec![format!("{}", client_index), format!("{}", request), format!("{}", time.format("%H:%M:%S %d.%m.%y"))].into_iter(), &quit)
                    },
        		    RealmsProtocol::Connect(_) => {
        		    	Row::StyledData(vec![format!("{}", client_index), format!("Connect"), format!("{}", time.format("%H:%M:%S %d.%m.%y"))].into_iter(), &done)
        		    },
        		    _ => Row::StyledData(vec![format!("{}", client_index), format!("{}", request), format!("{}", time.format("%H:%M:%S %d.%m.%y"))].into_iter(), &style)
        		}
            });

            Table::new(
                ["cId", "request", "time"].into_iter(),
                requests
            ).block(Block::default().title("Requests").borders(Borders::ALL))
                .header_style(Style::default().fg(Color::Yellow))
                .widths(&[4, 48, 17])
                .render(t, &chunks[0]);


        	let clients = clients.iter().enumerate().map(|(index, (_, client))| {
        		match client.connected {
        		    true => Row::StyledData(
	                    vec![format!("{}", index), format!("{}", client.id), format!("{}", client.completed_variants.len()), format!("{}", client.time.format("%H:%M:%S %d.%m.%y"))].into_iter(),
	                    &done
                	),
        		    false => Row::StyledData(
	                    vec![format!("{}", index), format!("{}", client.id), format!("{}", client.completed_variants.len()), format!("{}", client.time.format("%H:%M:%S %d.%m.%y"))].into_iter(),
	                    &style
	                ),
        		}
            });

            Table::new(
                ["cId", "uuid", "realms", "time"].into_iter(),
                clients
            ).block(Block::default().title("Clients").borders(Borders::ALL))
                .header_style(Style::default().fg(Color::Yellow))
                .widths(&[4, 42, 6, 17])
                .render(t, &chunks[1]);


        	let realms = realms.iter().rev().map(|realm| {
        		match realm.done {
        		    true => {
        		    	Row::StyledData(
		                    vec![format!("{}", realm.id), format!("{}", realm.age), format!("{}", realm.title)].into_iter(),
		                    &done
		                )
        		    },
        		    false => {
        		    	Row::StyledData(
		                    vec![format!("{}", realm.id), format!("{}", realm.age), format!("{}", realm.title)].into_iter(),
		                    &style
		                )
        		    },
        		}
        		
            });

            Table::new(
                ["rId", "age", "title"].into_iter(),
                realms
            ).block(Block::default().title("Realm").borders(Borders::ALL))
                .header_style(Style::default().fg(Color::Yellow))
                .widths(&[4, 5, 60])
                .render(t, &chunks[2]);
        });
    // end Groupd::default()

	t.draw()
}