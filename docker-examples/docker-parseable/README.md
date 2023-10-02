# Sologger with Logstash and Parseable

**Overview**

This example shows how to run the sologger service with Logstash and Parseable. Be aware, this example is set up to store all the log data locally. So if you are listening to all programs with the log level set to info, you will potentially receive a lot of data.
This example is set up to listen to all programs, but only log those that contain errors. If you want to listen to specific programs, then update the program IDs in the sologger-config.json file.
NOTE: This Parseable example is not setup to handle all logging all the Program logs at info level.

**Run**

```shell
docker compose up
```

**View Logs**

Go to http://localhost:8000/login and login with the user name admin and the password admin. The click on the logs link in the left hand navigation bar (http://localhost:8000/logstashlogs/logs). You should see a list of logs. Click on the log to see the details.