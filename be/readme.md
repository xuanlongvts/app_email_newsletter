1. Api curl subscriptions
   curl -X POST -d 'name=Long Le' -d 'email=longle@gmail.com' http://127.0.0.1:8000/subscriptions
2. Check packages unsed
   sudo cargo +nightly udeps
3. Test log with bunyan
   sudo TEST_LOG=true cargo test health_check_works | bunyan
4. Run with prettier format
   sudo cargo run | bunyan


5. Docker build
   sudo docker build --tag email_newsletter --file Dockerfile .
