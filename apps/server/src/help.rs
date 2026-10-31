pub fn display_help_message() {
  println!(
    "Usage: ./watchtower_server [OPTIONS]\n\
    Options:\n\
    \t-h: Displays this message\n\
    \t--password PASSWORD: [OPTIONAL] The password for the server that the agents must match. Defaults to an empty string, but it is heavily recommended to set a strong password.\n\
    \t--threads THREAD_COUNT: [OPTIONAL] The number of threads the server will use to handle connections. Must be greater than 0. Defaults to 4.\
    "
  );
}
