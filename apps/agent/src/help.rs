pub fn display_help_message() {
  println!(
    "Usage: ./watchtower_agent [OPTIONS]\n\
    Options:\n\
    \t-h: Displays this message\n\
    \t--watchtower-address \"ADDRESS:PORT\": The address and port of the watchtower server to connect to. By default, the watchtower server will run on 127.0.0.1:9824, but this option is still mandatory. The server will print the address it's running on when it starts.\n\
    \t--password PASSWORD: [OPTIONAL] Uses PASSWORD to authenticate with the server. Set this to the password configured for the watchtower server. Defaults to an empty string.\n\
    \t--test-instances NUM_INSTANCES: [OPTIONAL] When set, runs NUM_INSTANCES test instances to communicate with the watchtower server. They will send mock data in lieu of your actual device's statistics. Defaults to 0, which tells the agent to broadcast the device data normally.\
    "
  );
}
