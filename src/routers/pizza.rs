use crate::db::data_trait::pizza_data_trait::PizzaDataTrait;
use crate::db::database::Database;
use crate::error::pizza_error::PizzaError;
use crate::models::pizza::{BuyPizzaRequest, Pizza, UpdatePizza, UpdatePizzaURL};
use actix_web::web::{Data, Json, Path};
use actix_web::{get, patch, post};
use uuid::Uuid;
use validator::Validate;

pub fn pizza_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_pizzas);
    cfg.service(buy_pizza);
    cfg.service(update_pizza);
}

#[get("/pizzas")]
async fn get_pizzas(db: Data<Database>) -> Result<Json<Vec<Pizza>>, PizzaError> {
    let pizzas = Database::get_all_pizzas(&db).await;
    match pizzas {
        Some(pizzas) => Ok(Json(pizzas)),
        None => Err(PizzaError::NoPizzasFound),
    }
}

#[post("/pizzas/buy")]
async fn buy_pizza(
    body: Json<BuyPizzaRequest>,
    db: Data<Database>,
) -> Result<Json<Pizza>, PizzaError> {
    let is_valid = body.validate();
    match is_valid {
        Ok(_) => {
            let pizza_name = body.pizza_name.clone();
            let mut buffer = Uuid::encode_buffer();
            let new_uuid = Uuid::new_v4().simple().encode_lower(&mut buffer);

            let new_pizza =
                Database::add_pizza(&db, Pizza::new(String::from(new_uuid), pizza_name)).await;

            match new_pizza {
                Some(pizza) => Ok(Json(pizza)),
                None => Err(PizzaError::PizzaCreationFailure),
            }
        }
        Err(_) => Err(PizzaError::PizzaCreationFailure),
    }
}

#[patch("/pizzas/{uuid}")]
async fn update_pizza(
    update_pizza_url: Path<UpdatePizzaURL>,
    body: Json<UpdatePizza>,
    db: Data<Database>,
) -> Result<Json<Pizza>, PizzaError> {
    let is_valid = body.validate();
    match is_valid {
        Ok(_) => {
            let updated_pizza =
                Database::update_pizza(&db, update_pizza_url.uuid.clone(), body.pizza_name.clone())
                    .await;

            match updated_pizza {
                Some(pizza) => Ok(Json(pizza)),
                None => Err(PizzaError::NoSuchPizzaFound),
            }
        }
        Err(_) => Err(PizzaError::NoSuchPizzaFound),
    }
}
