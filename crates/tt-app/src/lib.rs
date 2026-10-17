//! The imperative shell around the core: owns `State` and the store behind one lock,
//! executes `Command`s and publishes `Event`s. Shared by every driver.
