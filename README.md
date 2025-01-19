This is a proof-of-concept implementation of the proposed blockchain architecture from the paper "Redefining Hybrid Blockchains: A balanced architecture" by Syed Ibrahim Omer. This is research code and may contain issues and lack certain functionalities.

**DO NOT USE THIS PROJECT IN A PRODUCTION ENVIRONMENT** 

The project contains the following directories:
-   node_pod: contains the source code for the validator node pods
-   master_pod: contains the source code for the master node pod
-   transaction_pod: contains the source code for the transaction simulator pod
-   kafka_deployment: contains the stateful sets, configMaps, service YAML files for the Kafka deployment on Kubernetes
-   node_deployment: contains the configmap and deployment YAML files for the node pod, master pod and transaction pod
-   single_node_test: contains the performance data obtained from the single node experiment
-   3_node_test: contains the performance data obtained from the 3-node experiment
-   4_node_test: contains the performance data obtained from the 4-node experiment

> **NOTE:** All the images are available publicly at DockerHub

## Dependency to replicate the experiment
1. minikube (installation: [https://minikube.sigs.k8s.io/docs/start/?arch=%2Flinux%2Fx86-64%2Fstable%2Fbinary+download](https://minikube.sigs.k8s.io/docs/start/?arch=%2Flinux%2Fx86-64%2Fstable%2Fbinary+download))

## Steps to replicate the experiment

 1. Clone the repository and navigate to the repository in the terminal.
 
 3. Start a minikube cluster (adjust vm-driver, cpu and memory resources based on your configuration)

> 
	minikube start --vm-driver=docker --cpus=10 --memory=30000

 

 3. Deploy the Kafka cluster on Kubernetes
 

> 
	kubectl apply -f kafka_deployment
	kubectl get pods -w



4. Apply all the deployments and ConfigMaps and wait for the containers to finish creating (should take about 5 minutes or a bit more)

> 
	kubectl apply -f node_deployment/conf
	kubectl apply -f node_deployment/
	kubectl get pods -w
	
5. Open a new terminal window, create the Kafka topics and listen on the Commit channel to listen for committed blocks

>  
	kubectl exec -it kafka-0 -- chmod +x /usr/local/bin/scripts/*.sh
	kubectl exec -it kafka-0  -- bash /usr/local/bin/scripts/create-topic.sh \
	Stakes Validators Primary Preprepare Prepare Commit Status Transactions Users
	kubectl exec -it kafka-0 -- /opt/bitnami/kafka/bin/kafka-console-consumer.sh \ 
	--bootstrap-server localhost:9092 --topic Commit --from-beginning
	
**NOTE:** If you want to delete all the topics run this:

> 
	kubectl exec -it kafka-0  -- bash /usr/local/bin/scripts/delete-topic.sh


 6. Extract the performance data from the node pods at any time after the blocks start committing
>

     kubectl cp <node_id>:data.csv <copy_path/file_name.csv>

**NOTE:** node_id can be found by running `kubectl get pods` and copying from the NAME field

7. Delete all the pods and exit the experiment 

> 
	kubectl delete deployment master-pod
	kubectl delete deployment node-pod
	kubectl delete job tx-pod-job
	minikube stop

 ## Configurations
- Adjust the node_pod and master_pod configurations by modifying their configMap

> **NOTE:** The block size in the node_pod configMap must be one of the order of 2 	since the merkle root can only be made for those number of leaves. So numbers like 64, 128, 256, 512 etc. Also the block size can not be greater than 512 as there are limitations on the memory size of vectors and hashers

- Adjust the amount of CPU and memory resources in the deployments' yaml files. You can also adjust the number of replicas for the pod there that will scale the number of nodes. For instance, replicas set to 4 means 4 validator nodes.

> **NOTE:** Ensure that you adjust the number of validators (in the master_node configMap) to be the same as the replicas. This is because for a small number of validators all the nodes must participate or there may be an error. 

