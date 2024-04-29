#!/usr/bin/env bash

declare ktl
ktl=$(which kubecolor 2> /dev/null || echo "kubectl")

# setup cloudnative-pg
$ktl cnpg install generate --watch-namespace example | $ktl apply --server-side -f -

webhook='cnpg-mutating-webhook-configuration'

$ktl wait --timeout 90s --for jsonpath='{.metadata.name}'=$webhook --namespace cnpg-system mutatingwebhookconfigurations.admissionregistration.k8s.io/$webhook

# setup ingress-nginx
$ktl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.10.1/deploy/static/provider/kind/deploy.yaml

# setup docker images
kind load docker-image --name winvoice winvoice-server:0.6.4

# apply
$ktl apply -Rf namespace.yaml,database/,server/
$ktl create -n example secret tls winvoice.tls --key key.pem --cert cert.pem
