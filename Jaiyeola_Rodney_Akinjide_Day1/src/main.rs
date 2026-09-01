use axum::Json;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MenuItem {
    name: String,
    price: u32, // naira
}

#[derive(Debug, Serialize, Deserialize)]
struct OrderLine {
    food: String,
    quantity: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Order {
    items: Vec<OrderLine>,
}

impl Order {
    fn new() -> Self {
        Order { items: Vec::new() }
    }

    fn add(&mut self) -> io::Result<()> {
        println!("please enter the item you would like to buy");
        let mut food = String::new();
        io::stdin().read_line(&mut food)?;
        let food = food.trim().to_string();

        println!("please enter the quantity");
        let mut qty = String::new();
        io::stdin().read_line(&mut qty)?;

        let quantity: u32 = match qty.trim().parse() {
            Ok(n) => n,
            Err(_) => {
                println!("invalid quantity entered");
                return Ok(());
            }
        };

        self.items.push(OrderLine { food, quantity });
        Ok(())
    }

    fn list(&self) -> &[OrderLine] {
        &self.items
    }
}

async fn json_requests(Json(payload): Json<Order>) -> Json<String> {
    println!("{:?}", payload);
    Json("Order successfully received".to_string())
}

fn main() {
    let mut order = Order::new();
    order.add().expect("failed to read stdin");

    for line in order.list() {
        println!("{} x{}", line.food, line.quantity);
    }
}
