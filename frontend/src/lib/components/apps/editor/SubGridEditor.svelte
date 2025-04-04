<script lang="ts">
	import { push } from '$lib/history'
	import { classNames } from '$lib/utils'
	import { createEventDispatcher, getContext, onDestroy } from 'svelte'
	import { twMerge } from 'tailwind-merge'
	import { gridColumns, isFixed, toggleFixed } from '../gridUtils'
	import Grid from '../svelte-grid/Grid.svelte'
	import type { AppEditorContext, AppViewerContext, GridItem } from '../types'
	import {
		expandGriditem,
		findGridItem,
		findGridItemParentGrid,
		insertNewGridItem,
		isContainer,
		maxHeight,
		selectId,
		subGridIndexKey
	} from './appUtils'
	import Component from './component/Component.svelte'
	import ComponentWrapper from './component/ComponentWrapper.svelte'
	import GridViewer from './GridViewer.svelte'
	import GridEditorMenu from './GridEditorMenu.svelte'

	export let containerHeight: number | undefined = undefined
	export let containerWidth: number | undefined = undefined
	let classes = ''

	export { classes as class }
	export let style = ''
	export let noPadding = false
	export let noYPadding = false
	export let subGridId: string
	export let visible: boolean = true
	export let id: string
	export let shouldHighlight: boolean = true

	const dispatch = createEventDispatcher()

	const {
		app,
		connectingInput,
		selectedComponent,
		focusedGrid,
		mode,
		parentWidth,
		breakpoint,
		allIdsInPath,
		worldStore
	} = getContext<AppViewerContext>('AppViewerContext')

	const editorContext = getContext<AppEditorContext>('AppEditorContext')

	let isActive = false
	let sber = editorContext?.componentActive?.subscribe((x) => (isActive = x))

	let everVisible = visible

	$: visible && !everVisible && (everVisible = true)

	onDestroy(() => {
		sber?.()
	})
	$: highlight = id === $focusedGrid?.parentComponentId && shouldHighlight

	const onpointerdown = (e) => {
		dispatch('focus')
	}

	function selectComponent(e: PointerEvent, id: string) {
		if (!$connectingInput.opened) {
			selectId(e, id, selectedComponent, $app)
		}
	}

	function lock(dataItem: GridItem) {
		let fComponent = findGridItem($app, dataItem.id)
		if (fComponent) {
			fComponent = toggleFixed(fComponent)
		}
		$app = $app
	}

	let container: HTMLElement | undefined = undefined

	$: maxRow = maxHeight($app.subgrids?.[subGridId] ?? [], containerHeight ?? 0, $breakpoint)

	export function moveComponentBetweenSubgrids(
		componentId: string,
		parentComponentId: string,
		subGridIndex: number,
		position?: { x: number; y: number }
	) {
		// Find the component in the source subgrid
		const component = findGridItem($app, componentId)

		if (!component) {
			return
		}

		let parentGrid = findGridItemParentGrid($app, component.id)

		if (parentGrid) {
			$app.subgrids &&
				($app.subgrids[parentGrid] = $app.subgrids[parentGrid].filter(
					(item) => item.id !== component?.id
				))
		} else {
			$app.grid = $app.grid.filter((item) => item.id !== component?.id)
		}

		const gridItem = component

		insertNewGridItem(
			$app,
			(id) => ({ ...gridItem.data, id }),
			{ parentComponentId: parentComponentId, subGridIndex: subGridIndex },
			Object.fromEntries(gridColumns.map((column) => [column, gridItem[column]])),
			component.id,
			position,
			undefined,
			undefined,
			undefined,
			true
		)

		// Update the app state
		$app = { ...$app }

		if (parentGrid) {
			$focusedGrid = {
				parentComponentId,
				subGridIndex
			}

			$selectedComponent = [parentComponentId]
		} else {
			$focusedGrid = undefined
		}
	}

	export function moveToRoot(componentId: string, position?: { x: number; y: number }) {
		// Find the component in the source subgrid
		const component = findGridItem($app, componentId)

		if (!component) {
			return
		}

		let parentGrid = findGridItemParentGrid($app, component.id)

		if (parentGrid) {
			$app.subgrids &&
				($app.subgrids[parentGrid] = $app.subgrids[parentGrid].filter(
					(item) => item.id !== component?.id
				))
		} else {
			$app.grid = $app.grid.filter((item) => item.id !== component?.id)
		}

		const gridItem = component

		insertNewGridItem(
			$app,
			(id) => ({ ...gridItem.data, id }),
			undefined,
			Object.fromEntries(gridColumns.map((column) => [column, gridItem[column]])),
			component.id,
			position,
			undefined,
			undefined,
			undefined,
			true
		)

		// Update the app state
		$app = { ...$app }
	}
</script>

{#if everVisible || $app.eagerRendering}
	<div
		class="translate-x-0 translate-y-0 w-full subgrid {visible
			? 'visible'
			: 'invisible h-0 overflow-hidden'}"
		bind:this={container}
		on:pointerdown={onpointerdown}
	>
		<div
			class={twMerge(
				$allIdsInPath.includes(id) && $mode == 'dnd' ? 'overflow-visible' : 'overflow-auto',
				noYPadding ? '' : 'py-2',
				classes ?? '',
				noPadding ? 'px-0' : 'px-2'
			)}
			style="{containerHeight ? `height: ${containerHeight - 2}px;` : ''} {style ?? ''}"
		>
			{#if $mode !== 'preview'}
				<div
					class={highlight
						? `outline !outline-dashed outline-2 min-h-full ${
								isActive && !$selectedComponent?.includes(id)
									? 'outline-orange-600'
									: 'outline-gray-400 dark:outline-gray-600'
						  }`
						: ''}
				>
					<Grid
						allIdsInPath={$allIdsInPath}
						items={$app.subgrids?.[subGridId] ?? []}
						on:redraw={(e) => {
							push(editorContext?.history, $app)
							if ($app.subgrids) {
								$app.subgrids[subGridId] = e.detail
							}
						}}
						selectedIds={$selectedComponent}
						let:dataItem
						let:overlapped
						let:moveMode
						let:componentDraggedId
						scroller={container}
						parentWidth={$parentWidth - 17}
						{containerWidth}
						on:dropped={(e) => {
							const { id, overlapped, x, y } = e.detail

							if (!overlapped) {
								moveToRoot(id, { x, y })
							} else {
								const overlappedComponent = findGridItem($app, overlapped)

								if (overlappedComponent && !isContainer(overlappedComponent.data.type)) {
									return
								}
								if (!overlapped) {
									return
								}

								if (id === overlapped) {
									return
								}

								moveComponentBetweenSubgrids(
									id,
									overlapped,
									subGridIndexKey(overlappedComponent?.data?.type, overlapped, $worldStore),
									{ x, y }
								)
							}
						}}
						disableMove={!!$connectingInput.opened}
					>
						<ComponentWrapper
							id={dataItem.id}
							type={dataItem.data.type}
							class={classNames(
								'h-full w-full center-center',
								$selectedComponent?.includes(dataItem.id) ? 'active-grid-item' : '',
								'top-0'
							)}
						>
							<GridEditorMenu id={dataItem.id}>
								<Component
									{overlapped}
									fullHeight={dataItem?.[$breakpoint === 'sm' ? 3 : 12]?.fullHeight}
									render={visible}
									component={dataItem.data}
									selected={Boolean($selectedComponent?.includes(dataItem.id))}
									locked={isFixed(dataItem)}
									on:lock={() => lock(dataItem)}
									on:expand={() => {
										const parentGridItem = findGridItem($app, id)

										if (!parentGridItem) {
											return
										}

										$selectedComponent = [dataItem.id]
										push(editorContext?.history, $app)

										expandGriditem(
											$app.subgrids?.[subGridId] ?? [],
											dataItem.id,
											$breakpoint,
											parentGridItem
										)
										$app = $app
									}}
									on:fillHeight={() => {
										const gridItem = findGridItem($app, dataItem.id)
										const b = $breakpoint === 'sm' ? 3 : 12

										if (gridItem?.[b]) {
											gridItem[b].fullHeight = !gridItem[b].fullHeight
										}
										$app = $app
									}}
									{moveMode}
									{componentDraggedId}
								/>
							</GridEditorMenu>
						</ComponentWrapper>
					</Grid>
				</div>
			{:else}
				<GridViewer
					allIdsInPath={$allIdsInPath}
					items={$app.subgrids?.[subGridId] ?? []}
					let:dataItem
					breakpoint={$breakpoint}
					parentWidth={$parentWidth - 17}
					{containerWidth}
					{maxRow}
				>
					<!-- svelte-ignore a11y-click-events-have-key-events -->
					<div
						on:pointerdown|stopPropagation={(e) => selectComponent(e, dataItem.id)}
						class={classNames('h-full w-full center-center', 'top-0')}
					>
						<Component
							fullHeight={dataItem?.[$breakpoint === 'sm' ? 3 : 12]?.fullHeight}
							render={visible}
							component={dataItem.data}
							selected={Boolean($selectedComponent?.includes(dataItem.id))}
							locked={isFixed(dataItem)}
						/>
					</div>
				</GridViewer>
			{/if}
		</div>
	</div>
{:else if $app.lazyInitRequire == undefined}
	{#each $app?.subgrids?.[subGridId] ?? [] as item}
		<Component selected={false} fullHeight={false} render={false} component={item.data} />
	{/each}
{/if}
