# timeline

Timeline is a project 100% written in rust. Timeline aims to collect the most data about your day to day life and organize it in a timeline. This is archived through plugins. Keep a lookout for timeline*plugin*\* repositories. Plugins can be installed by cloning their repository into the (a) plugins folder in the root of this repository.

# run timeline

1. build the frontend by running trunk build --release in the frontend directory
2. run the package generator with cargo run
3. to run the timeline server simply run cargo run --release in the server directory

# experiences
1. enable the experiences feature 
2. create the "experiences_location.txt" file and write the path of the experiences project to it
3. add the experiences url field to the config