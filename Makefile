docker: check-env
	docker build -t gcr.io/${PROJECT_ID}/ingredients_bot .
docker-upload: check-env
	gcloud docker -- push gcr.io/${PROJECT_ID}/ingredients_bot

cloud-run: docker docker-upload

clean:
	cargo clean

check-env:
ifndef PROJECT_ID
	$(error PROJECT_ID is undefined)
endif
