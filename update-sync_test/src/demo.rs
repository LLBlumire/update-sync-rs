use update_sync::{derive, UpdateSync};

/// Data that our server will be syncronising between users    
#[derive(derive::UpdateSync, Clone, PartialEq, Default, Debug)]
struct Record {
    name: String,
    year_of_birth: u32,
    month_of_birth: u32,
    day_of_birth: u32,
    password: String,
}

/// A fake server to demonstrate how data might be updated and syncronised
#[derive(Default, Debug)]
struct Server {
    /// Our server is storing information about a single user, in a more complex example this might be a map,
    /// it's keeping it in an option so we can take ownership and replace it
    user: Option<Record>,
}
impl Server {
    /// Updates our data with new data sent in by the client
    ///
    /// This might be triggered by a HTTP request, or some other server-client protocol, it's return would be our response
    fn update(&mut self, client_last_known: Record, client_new: Record) -> Record {
        // `self.user` should always be set for the demo, so we'll just crash if it isn't
        let current_version = self.user.take().unwrap();

        // We pass our data to update sync in the order it should exist chronologically
        // `client_last_known` will be some old data from last time the client synced
        // `current_version` will be the version our server currently knows of
        // `client_new` is the new version the client would like to set
        let new = UpdateSync::update_sync(client_last_known, current_version, client_new);

        // `self.user` needs to be replaced
        self.user = Some(new.clone());

        // And `new` needs to go back to the client for their new synced version of the data
        new
    }

    /// Provides data to a client that wants to sync without sending data
    ///
    /// This might be triggered by a HTTP request, or some other server-client protocol, it's return would be our response
    fn sync(&mut self) -> Record {
        // Again, in a more complex example we should make sure our state isn't invalid
        self.user.clone().unwrap()
    }

    /// This is just here for our own testing!
    #[cfg(test)]
    fn assert_user_is(&self, user: &Record) {
        assert_eq!(self.user.as_ref(), Some(user))
    }
}

/// Our client will store two coppies of the data
#[derive(Default)]
struct Client {
    /// The version of the data known to the client last time it synced
    user_last_synced: Record,
    /// The live, editable data for the client
    user: Record,
}
impl Client {
    /// Syncronises data between the client and server
    ///
    /// This would send and receive data over some network protocol in reality
    fn sync(&mut self, server: &mut Server) {
        self.user = server.sync();
        self.user_last_synced = self.user.clone();
    }

    /// Sends new changed data to the server, and pulls our update
    ///
    /// This would send and receive data over some network protocol in reality
    fn send(&mut self, server: &mut Server) {
        self.user = server.update(self.user_last_synced.clone(), self.user.clone());
        self.user_last_synced = self.user.clone();
    }
}

#[test]
fn full_demo() {
    // Lets set up the clients and server for our demo
    let ref mut server = Server::default();
    server.user = Some(Record::default());
    let ref mut client_1 = Client::default();
    let ref mut client_2 = Client::default();
    client_1.sync(server);
    client_2.sync(server);

    // Lets start with some basic synced operations, our first client
    // is goint to set up the data to start with, and send it to the server
    client_1.user.name = "Lucille Blumire".into();
    client_1.user.year_of_birth = 1998;
    client_1.user.month_of_birth = 9;
    client_1.user.day_of_birth = 23;
    client_1.user.password = "password".into();
    client_1.send(server);

    // And a while passes, so client 2 ends up syncing
    client_2.sync(server);

    // Now for something a bit more complicated, both of them are going to make an edit!
    // The first client notices that my date of birth is wrong, so fixes it
    client_1.user.year_of_birth = 1997;

    // The second client knows for certain I wouldn't use a password that insecure
    client_2.user.password = "password1".into();
    // much better

    // Now they end up sending in their updates, in some order
    client_1.send(server);
    client_2.send(server);
    server.assert_user_is(&Record {
        name: "Lucille Blumire".into(),
        year_of_birth: 1997,
        month_of_birth: 9,
        day_of_birth: 23,
        password: "password1".into(),
    });

    // Now note, that client 2 will have been synced when they sent data, but client one is still out of sync!
    // as far as they are concerned, they have the best data possible. They are going to submit some changes
    // First, they realise the day of birth that has been set is wrong
    client_1.user.day_of_birth = 24;
    // And they also think the password is still "password", so lets have them set their own rendition
    client_1.user.password = "Password1!".into();
    // even more secure!

    // At the same time, client 2 wants to fix my name
    client_2.user.name = "Lucille Lillian Blumire".into();
    // and is gonna send their changes in first this time (so client_1 is still very out of date)
    client_2.send(server);
    // and now client 1 is **finally** going to sync.
    client_1.send(server);

    server.assert_user_is(&Record {
        name: "Lucille Lillian Blumire".into(),
        year_of_birth: 1997,
        month_of_birth: 9,
        day_of_birth: 24,
        password: "Password1!".into(),
    });
}
