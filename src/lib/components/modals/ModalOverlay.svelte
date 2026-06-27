<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title,
		children,
		onclose
	}: {
		title: string;
		children: Snippet;
		onclose: () => void;
	} = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-overlay" onclick={onclose}>
	<div class="modal-body" onclick={(e) => e.stopPropagation()}>
		<header class="modal-header">
			<h2 class="modal-title">{title}</h2>
			<button class="close-button" onclick={onclose} aria-label="閉じる">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="20"
					height="20"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<line x1="18" y1="6" x2="6" y2="18"></line>
					<line x1="6" y1="6" x2="18" y2="18"></line>
				</svg>
			</button>
		</header>
		<div class="modal-content">
			{@render children()}
		</div>
	</div>
</div>

<style>
	.modal-overlay {
		position: fixed;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		background-color: var(--modal-overlay);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
		backdrop-filter: blur(2px);
	}

	.modal-body {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: 12px;
		box-shadow: 0 10px 25px -5px var(--modal-shadow), 0 8px 10px -6px var(--modal-shadow);
		width: 90%;
		max-width: 500px;
		max-height: 85vh;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		animation: modal-enter 0.2s ease-out;
	}

	@keyframes modal-enter {
		from {
			opacity: 0;
			transform: scale(0.95);
		}
		to {
			opacity: 1;
			transform: scale(1);
		}
	}

	.modal-header {
		padding: 1rem 1.25rem;
		border-bottom: 1px solid var(--border);
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.modal-title {
		margin: 0;
		font-size: 1.125rem;
		font-weight: 600;
		color: var(--text-primary);
	}

	.close-button {
		background: transparent;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		padding: 4px;
		border-radius: 6px;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 0.2s;
	}

	.close-button:hover {
		background-color: var(--bg-elevated);
		color: var(--text-primary);
	}

	.modal-content {
		padding: 1.25rem;
		overflow-y: auto;
		flex: 1;
	}
</style>
