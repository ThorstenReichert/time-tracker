import { mount, type ComponentMountingOptions } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import PrimeVue from "primevue/config";
import Toast from "primevue/toast";
import ToastService from "primevue/toastservice";
import { defineComponent, h, type Component } from "vue";

/**
 * Mounts a main-window component the way main.ts sets the app up: PrimeVue, the toast service
 * and a fresh Pinia. It's attached to the page, so dialogs and overlays (teleported to the
 * body) can be found with `document`.
 */
export function mountInApp<C extends Component>(component: C, options: ComponentMountingOptions<C> = {}) {
  const pinia = createPinia();
  setActivePinia(pinia);
  return mount(component, {
    ...options,
    attachTo: document.body,
    global: { ...options.global, plugins: [pinia, PrimeVue, ToastService, ...(options.global?.plugins ?? [])] },
  });
}

/** `component` next to a Toast, as App.vue renders it, so failure toasts show up on the page. */
export function withToast(component: Component) {
  return defineComponent({ render: () => [h(component), h(Toast)] });
}
