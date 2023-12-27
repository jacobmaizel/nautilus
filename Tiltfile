load('ext://helm_resource', 'helm_resource', 'helm_repo')
load('ext://namespace', 'namespace_create', 'namespace_inject')
update_settings(k8s_upsert_timeout_secs=120)

namespace_create('trainton-dev', annotations=[ 'linkerd.io/inject: enabled' ])


local_resource("Create minikube storage claims", "minikube ssh  \"sudo mkdir /prometheus-data && sudo mkdir /grafana-data &&  sudo chmod 777 /grafana-data && sudo chmod 777 /prometheus-data\"", auto_init=True, )

# /* -------------------------------- Nautilus -------------------------------- */
docker_build('nautilus', '.', dockerfile='docker/Dockerfile.dev', live_update=[sync('.', '/app')], entrypoint='cargo watch -x run')
k8s_yaml('kubernetes/server/config.yaml')
k8s_yaml('kubernetes/server/dev-deployment.yaml')
k8s_resource('nautilus', port_forwards='5050:5050')



# /* --------------------------------- Jaeger --------------------------------- */
# http://localhost:16686
#! TODO: SETUP JAEGER OPERATOR HERE https://www.jaegertracing.io/docs/1.16/operator/
helm_repo('jaegertracing', 'https://jaegertracing.github.io/helm-charts')
helm_resource('jaeger', 'jaegertracing/jaeger', resource_deps=['jaegertracing'], flags=["--values=kubernetes/jaeger/values.yaml"], port_forwards=["16686:16686", "14269:14269"], namespace='trainton-dev')
k8s_yaml('kubernetes/jaeger/service.yaml')


helm_repo('prometheus-community-repo', 'https://prometheus-community.github.io/helm-charts')
helm_resource(name="kube-prometheus-stack", chart="prometheus-community-repo/kube-prometheus-stack", namespace='trainton-dev', port_forwards=["9090:9090"], flags=['--values=kubernetes/kube-prometheus-stack/values.yaml'])



# /* ------------------------------ CloudNativePG ----------------------------- */

helm_repo(name='cnpg', url='https://cloudnative-pg.github.io/charts')
helm_resource(name='cloudnative-pg', chart='cnpg/cloudnative-pg', namespace='trainton-dev',resource_deps=[])
k8s_yaml('kubernetes/cloudnativepg/user-secret.yaml')


# /* ----------------------------- Local Resources ---------------------------- */

local_resource(
  name='Port forward Grafana',
  serve_cmd='kubectl port-forward svc/kube-prometheus-stack-grafana  3000:3000 -n trainton-dev')

local_resource(name="Start CloudPG Cluster", serve_cmd= "while ! kubectl apply -f kubernetes/cloudnativepg/cluster.yaml; do echo \"Command failed, retrying...\"; sleep 5; done")


local_resource(
  name='Port forward CloudPG', serve_cmd='kubectl port-forward svc/cloudpg-cluster-rw  5432:5432 -n trainton-dev'
  )



# /* ----------------------------- Archived stuff ----------------------------- */

# namespace_create('monitoring', annotations=[ 'linkerd.io/inject: enabled' ])

# namespace_create('linkerd')
# namespace_create('linkerd-viz')
# namespace_create('linkerd-buoyant')

# Helm

# /* -------------------------------------------------------------------------- */
# /*                                  Linkerd                                   */
# /* -------------------------------------------------------------------------- */
#! https://linkerd.io/2.14/tasks/install-helm/

# /* ---------------------------  Chart 0: linkerd repo init --------------------------- */
# helm_repo('linkerd', 'https://helm.linkerd.io/stable')

# /* -------------------------  Chart 1: linkerd-crds ------------------------ */
# helm_resource('linkerd-crds', 'linkerd/linkerd-crds', namespace='linkerd', pod_readiness="wait")

# /* ---------------------  Chart2: linkerd-control-plane -------------------- */
# helm_resource('linkerd-control-plane', 'linkerd/linkerd-control-plane', namespace='linkerd', flags=['--set-file=identityTrustAnchorsPEM=keys/ca.crt,identity.issuer.tls.crtPEM=keys/issuer.crt,identity.issuer.tls.keyPEM=keys/issuer.key', '--values=kubernetes/linkerd/values.yaml'], resource_deps=['linkerd'])

# /* --------------------------  Chart3: linkerd-viz ------------------------- */
# helm_resource('linkerd-viz', 'linkerd/linkerd-viz', resource_deps=['linkerd', 'linkerd-control-plane'], namespace='linkerd-viz', flags=['--values=kubernetes/linkerd/viz/values.yaml'])

# /* --------------------------  Chart4: linkerd-buoyant cloud ------------------------- */
# helm_repo('linkerd-buoyant','https://helm.buoyant.cloud')
# helm_resource('linkerd-buoyant-cloud', 'linkerd-buoyant/linkerd-buoyant', namespace='linkerd-buoyant', flags=['--set=metadata.agentName=cluster-1', '--values=kubernetes/linkerd/buoyant/values.yaml'], resource_deps=['linkerd-buoyant', 'linkerd-control-plane'])

# /* --------------------------------- Server --------------------------------- */


# /* ----------------------------------- DB ----------------------------------- */

# k8s_yaml('kubernetes/db/deploy.yaml')
# k8s_resource('postgres', resource_deps=['linkerd-control-plane'],port_forwards='5432:5432')


# /* -------------------------------- Prometheus ------------------------------- */
# http://localhost:9090/
# TODO: Setup prometheus + linkerd metrics https://linkerd.io/2.14/tasks/external-prometheus/
# helm_repo('prometheus-community' ,'https://prometheus-community.github.io/helm-charts')
# helm_resource('prometheus', 'prometheus-community/prometheus', resource_deps=['prometheus-community','linkerd-control-plane'], flags=['--set=server.persistentVolume.enabled=true,server.persistentVolume.storageClass=local-storage,server.persistentVolume.existingClaim=prometheus-pvc', '--values=kubernetes/prometheus/values.yaml'], port_forwards=["9090:9090"], namespace='trainton-dev')
# k8s_yaml('kubernetes/prometheus/config.yaml')

# # /* --------------------------------- Grafana -------------------------------- */
# # http://localhost:3000
# k8s_yaml('kubernetes/grafana/config.yaml')
# helm_repo('grafana-community' ,'https://grafana.github.io/helm-charts')
# helm_resource('grafana', 'grafana-community/grafana', resource_deps=['grafana-community','linkerd-control-plane'], flags=['--values=kubernetes/grafana/values.yaml'], port_forwards=["3000:80"],namespace='trainton-dev')




# /* ------------------------------ Prometheus + Grafana Community ----------------------------- */
# https://github.com/cloudnative-pg/charts/blob/main/charts/cloudnative-pg/monitoring/grafana-dashboard.json




# ,
# '--values=https://raw.githubusercontent.com/cloudnative-pg/cloudnative-pg/main/docs/src/samples/monitoring/kube-stack-config.yaml', 


# k8s_yaml('kubernetes/cloudnativepg/dev-service.yaml')
# local("sleep 10")
# k8s_yaml('kubernetes/cloudnativepg/cluster.yaml')

# k8s_resource("Create cloudnative-pg-cluster", "kubectl apply -f kubernetes/cloudnativepg/cluster.yaml", resource_deps=['cloudnative-pg'])

