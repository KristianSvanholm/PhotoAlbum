# Photo Album

## Requirements
* [Docker](https://docs.docker.com/desktop/install/linux-install/)
* [Docker-compose](https://docs.docker.com/compose/install/)

## Deployment
Deployment is done through our deployment script. Simply run the following command, which will take care of the rest.  
`$ sudo ./deploy.sh`  

## Accessing the files
The database along with the actual image files can be accessed in the home directory for the ROOT user of the system. 
All images are stored in their original formats in the `/album` directory. 
