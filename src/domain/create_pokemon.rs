#[cfg(test)]

struct Request {
    number: u16,
    name: String,
    types: Vec<String>,
}

enum Response {
    Ok(u16),
    BadRequest
}

fn execute(req: Request) -> Response {
    Response::BadRequest
}

mod test{
    use super::*;

    #[test]
    fn it_shoul_return_the_podemon_number_otherwise(){
        let number = 25;
        let req = Request {
            number,
            name: String::from("Pikachu"),
            types: vec![String::from("Eletric")],
        };


        let res = execute(req);

        match res {
            Response::Ok(res_number) => assert_eq!(res_number, number),
            _ => unreachable!(),
        };
    
    }

    #[test]
    fn it_should_return_a_bad_request_error_when_request_is_invalid() {
        let req = Request {
            number: 25, 
            name: String::from(""),
            types: vec![String::from("Eletric")]
        };

        let res = execute(req);

        match res {
            Response::BadRequest =>{},
            _ => unreachable!(),
        }
    }

}