use crate::db::database::Database;
use crate::models::pizza::Pizza;
use actix_web::web::Data;
use async_trait::async_trait;
use surrealdb::Error;

fn process_pizza_result(result: Result<Option<Pizza>, Error>, op: &str) -> Option<Pizza> {
    match result {
        Ok(Some(pizza)) => Some(pizza),
        Ok(None) => {
            eprintln!("No pizza was {} (got Ok(None))", op);
            None
        }
        Err(e) => {
            eprintln!("Error {} pizza: {:?}", op, e);
            None
        }
    }
}

#[async_trait]
pub trait PizzaDataTrait {
    async fn get_all_pizzas(db: &Data<Database>) -> Option<Vec<Pizza>>;
    async fn add_pizza(db: &Data<Database>, new_pizza: Pizza) -> Option<Pizza>;
    async fn update_pizza(
        db: &Data<Database>,
        pizza_uuid: String,
        pizza_name: String,
    ) -> Option<Pizza>;
}

#[async_trait]
impl PizzaDataTrait for Database {
    async fn get_all_pizzas(db: &Data<Database>) -> Option<Vec<Pizza>> {
        match db.client.select("pizza").await {
            Ok(all_pizzas) => Some(all_pizzas),
            Err(e) => {
                eprintln!("Error retrieving pizzas: {:?}", e);
                None
            }
        }
    }

    async fn add_pizza(db: &Data<Database>, new_pizza: Pizza) -> Option<Pizza> {
        let result = db
            .client
            .create(("pizza", new_pizza.uuid.clone()))
            .content(new_pizza)
            .await;

        process_pizza_result(result, "adding")
    }

    async fn update_pizza(
        db: &Data<Database>,
        pizza_uuid: String,
        pizza_name: String,
    ) -> Option<Pizza> {
        let result = db
            .client
            .update(("pizza", pizza_uuid.clone()))
            .content(Pizza::new(pizza_uuid, pizza_name))
            .await;

        process_pizza_result(result, "updating")
    }
}
