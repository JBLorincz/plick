use plick::Config;



pub fn generate_test_config() -> Config
{
    let config = 
         Config
         {
            dry_run: true,
            ..Config::default()
         };


    config
}
