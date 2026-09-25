import { computed, ref, watch } from 'vue';

export interface PaginationProps {
  page: number;
  total: number;
  itemsPerPage?: number;
  disabled?: boolean;
  label?: string;
}

export function usePagination(props: PaginationProps, onPage: (page: number) => void) {
  const pageCount = computed(() => Math.max(1, Math.ceil(props.total / (props.itemsPerPage ?? 20))));
  const targetPage = ref(String(props.page));
  watch(
    () => props.page,
    (page) => {
      targetPage.value = String(page);
    },
  );
  const validTarget = computed(
    () =>
      /^\d+$/.test(targetPage.value) && Number(targetPage.value) >= 1 && Number(targetPage.value) <= pageCount.value,
  );
  const jump = () => {
    if (!props.disabled && validTarget.value && Number(targetPage.value) !== props.page) {
      onPage(Number(targetPage.value));
    }
  };
  return { pageCount, targetPage, validTarget, jump };
}
